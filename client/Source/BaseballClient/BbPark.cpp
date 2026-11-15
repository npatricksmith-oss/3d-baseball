#include "BbPark.h"
#include "BbFrame.h"
#include "Async/ParallelFor.h"
#include "Components/DirectionalLightComponent.h"
#include "Components/SkyAtmosphereComponent.h"
#include "Components/SkyLightComponent.h"
#include "Components/StaticMeshComponent.h"
#include "Engine/DirectionalLight.h"
#include "Engine/ExponentialHeightFog.h"
#include "Engine/SkyLight.h"
#include "Engine/StaticMesh.h"
#include "Engine/StaticMeshActor.h"
#include "Engine/Texture2D.h"
#include "Engine/World.h"
#include "Kismet/GameplayStatics.h"
#include "Materials/Material.h"
#include "Materials/MaterialExpressionConstant.h"
#include "Materials/MaterialExpressionTextureSampleParameter2D.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "ProceduralMeshComponent.h"
#include "TextureResource.h"

DEFINE_LOG_CATEGORY_STATIC(LogBbPark, Log, All);

using BbFrameConv::ToUE;

namespace
{
	const double FT = 0.3048;
	const double R2 = 1.4142135623730951;

	// The palette (park-visuals step 2 gathers the whole park's in one place).
	const FLinearColor GrassLight(0.068f, 0.24f, 0.047f);
	const FLinearColor GrassDark(0.058f, 0.21f, 0.041f);
	const FLinearColor Dirt(0.4f, 0.235f, 0.125f);
	const FLinearColor Track(0.33f, 0.18f, 0.1f);
	const FLinearColor Chalk(0.9f, 0.9f, 0.88f);
	const FLinearColor Concrete(0.4f, 0.4f, 0.41f);
	const FLinearColor WallColor(0.02f, 0.08f, 0.2f);

	// The Official Baseball Rules' field diagram (Appendix 1-3), the same in every park.
	const double ChalkW = 4.0 / 12.0 * FT;       // foul lines
	const double BoxLineW = 3.0 / 12.0 * FT;     // the boxes' lines
	const double PlateHalf = 8.5 / 12.0 * FT;    // home plate is 17 in across
	const double BoxNear = PlateHalf + 0.5 * FT; // batter's box: 6 in off the plate,
	const double BoxFar = BoxNear + 4.0 * FT;    // 4 ft wide,
	const double BoxBack = PlateHalf - 3.0 * FT; // 6 ft long, centred on the plate's middle
	const double BoxFront = PlateHalf + 3.0 * FT;
	const double CatcherHalf = 43.0 / 24.0 * FT;
	const double CatcherBack = -8.0 * FT;
	const double PathHalf = 3.0 * FT;         // a base path's dirt, either side of the line
	const double CutoutR = 13.0 * FT;         // the dirt round home and round each base
	const double MoundR = 9.0 * FT;
	const double MoundH = 10.0 / 12.0 * FT;   // the rubber is 10 in above the plate
	const double SkinR = 95.0 * FT;           // the infield's outer arc, from the rubber's middle
	const double LaneFrom = 45.0 * FT;        // the runner's lane: the last half of the way to first
	const double CoachOff = 15.0 * FT, CoachLen = 20.0 * FT, CoachEnd = 5.0 * FT;
	const double OnDeckR = 2.5 * FT;
	const double BaseSide = 18.0 / 12.0 * FT; // bases are 18 in square
	// Groundskeeping that the rules leave to the park.
	const double TrackFence = 15.0 * FT;      // warning track under the outfield fence
	const double TrackWall = 10.0 * FT;       // and along the walls of foul ground
	const double Mow = 15.0 * FT;             // mown stripes in the outfield, running home to centre
	// The near picture of the ground: the infield, where the chalk has to be sharp.
	const double NearX0 = -28.0, NearY0 = -8.0, NearSize = 56.0;

	double Dist2(double X, double Y, const FVector2D& C)
	{
		return (X - C.X) * (X - C.X) + (Y - C.Y) * (Y - C.Y);
	}
}

FBbGround::FBbGround(const BbPark& InDesc)
	: Desc(InDesc)
{
	// The square that holds the park with a margin, on a 2 m grid.
	double MinX = 0, MaxX = 0, MinY = 0, MaxY = 0;
	for (uint32 I = 0; I < Desc.n_wall; ++I)
	{
		MinX = FMath::Min(MinX, (double)Desc.wall[I].from[0]);
		MaxX = FMath::Max(MaxX, (double)Desc.wall[I].from[0]);
		MinY = FMath::Min(MinY, (double)Desc.wall[I].from[1]);
		MaxY = FMath::Max(MaxY, (double)Desc.wall[I].from[1]);
	}
	const double Margin = 4.0;
	Size = 2.0 * FMath::CeilToDouble((FMath::Max(MaxX - MinX, MaxY - MinY) + 2.0 * Margin) / 2.0);
	X0 = 2.0 * FMath::FloorToDouble(((MinX + MaxX) * 0.5 - Size * 0.5) / 2.0);
	Y0 = 2.0 * FMath::FloorToDouble(((MinY + MaxY) * 0.5 - Size * 0.5) / 2.0);
	BaseL = FMath::Sqrt(Dist2(Desc.bases[0][0], Desc.bases[0][1], FVector2D::ZeroVector));
	for (int32 I = 0; I < 2; ++I)
	{
		PoleS[I] = FMath::Sqrt(Dist2(Desc.poles[I][0], Desc.poles[I][1], FVector2D::ZeroVector));
	}
	// The mound's circle is centred 18 in in front of the rubber; the infield's arc is struck
	// from the rubber's middle.
	MoundC = FVector2D(0.0, Desc.rubber_m - 1.5 * FT);
	SkinC = FVector2D(0.0, Desc.rubber_m + 0.25 * FT);
	// On-deck circles: 37 ft from home toward each dugout, two thirds of the way to the wall.
	double Clear = 50.0 * FT;
	for (uint32 I = 0; I < Desc.n_wall; ++I)
	{
		if (Desc.wall[I].kind == 4)
		{
			Clear = (FMath::Abs((double)Desc.wall[I].from[0]) - Desc.wall[I].from[1]) / R2;
		}
	}
	const double C = Clear * 0.66, S = FMath::Sqrt(FMath::Max(0.0, FMath::Square(37.0 * FT) - C * C));
	OnDeck[0] = FVector2D(-(S + C) / R2, (S - C) / R2);
	OnDeck[1] = FVector2D((S + C) / R2, (S - C) / R2);

	// Distance to the boundary on a 0.25 m grid: its sign from whether the point is inside the
	// loop, and the distance to the fence's pieces alone (for the warning track).
	Cells = FMath::RoundToInt32(Size / 0.25);
	const int32 Row = Cells + 1;
	Sd.SetNumUninitialized(Row * Row);
	Fd.SetNumUninitialized(Row * Row);
	const int32 NW = Desc.n_wall;
	ParallelFor(Row, [this, Row, NW](int32 J)
	{
		const double Y = Y0 + Size * J / Cells;
		for (int32 I = 0; I < Row; ++I)
		{
			const double X = X0 + Size * I / Cells;
			double Best = TNumericLimits<double>::Max(), BestFence = Best;
			bool bIn = false;
			for (int32 W = 0; W < NW; ++W)
			{
				const BbWallPiece& A = Desc.wall[W];
				const BbWallPiece& B = Desc.wall[(W + 1) % NW];
				const double Ax = A.from[0], Ay = A.from[1], Dx = B.from[0] - Ax, Dy = B.from[1] - Ay;
				const double T = FMath::Clamp(((X - Ax) * Dx + (Y - Ay) * Dy) / (Dx * Dx + Dy * Dy), 0.0, 1.0);
				const double D = FMath::Square(X - Ax - T * Dx) + FMath::Square(Y - Ay - T * Dy);
				Best = FMath::Min(Best, D);
				if (A.kind == 0)
				{
					BestFence = FMath::Min(BestFence, D);
				}
				if ((Ay > Y) != (B.from[1] > Y) && X < Ax + Dx * (Y - Ay) / Dy)
				{
					bIn = !bIn;
				}
			}
			Sd[J * Row + I] = (float)(FMath::Sqrt(Best) * (bIn ? 1.0 : -1.0));
			Fd[J * Row + I] = (float)FMath::Sqrt(BestFence);
		}
	});
}

void FBbGround::Lookup(double X, double Y, float& OutSd, float& OutFence) const
{
	const double U = FMath::Clamp((X - X0) / Size * Cells, 0.0, Cells - 1e-6);
	const double V = FMath::Clamp((Y - Y0) / Size * Cells, 0.0, Cells - 1e-6);
	const int32 I = (int32)U, J = (int32)V, Row = Cells + 1;
	const float Fu = (float)(U - I), Fv = (float)(V - J);
	const int32 K = J * Row + I;
	OutSd = FMath::Lerp(FMath::Lerp(Sd[K], Sd[K + 1], Fu), FMath::Lerp(Sd[K + Row], Sd[K + Row + 1], Fu), Fv);
	OutFence = FMath::Lerp(FMath::Lerp(Fd[K], Fd[K + 1], Fu), FMath::Lerp(Fd[K + Row], Fd[K + Row + 1], Fu), Fv);
}

float FBbGround::Inside(double X, double Y) const
{
	float S, F;
	Lookup(X, Y, S, F);
	return S;
}

double FBbGround::Height(double X, double Y) const
{
	// The mound (OBR Appendix 2): a level table round the rubber, 5 ft across and 34 in deep,
	// 10 in up; from 6 in in front of the rubber it falls an inch a foot toward the plate,
	// which brings it to the field at the circle's edge. The other sides fall to the circle's
	// edge the same way, steeper because the edge is nearer.
	const double E = MoundR - FMath::Sqrt(Dist2(X, Y, MoundC));
	if (E <= 0.0)
	{
		return 0.0;
	}
	const double Rub = Desc.rubber_m;
	const double Dx = FMath::Max(0.0, FMath::Abs(X) - 2.5 * FT);
	const double Dy = FMath::Max3(0.0, (Rub - 0.5 * FT) - Y, Y - (Rub + 28.0 / 12.0 * FT));
	const double D = FMath::Sqrt(Dx * Dx + Dy * Dy);
	return MoundH * E / (D + E);
}

FLinearColor FBbGround::Color(double X, double Y) const
{
	float SdHere, FenceHere;
	Lookup(X, Y, SdHere, FenceHere);
	if (SdHere < 0.f)
	{
		return Concrete;
	}
	const double Ax = FMath::Abs(X);
	// Along a foul line and off it into foul ground, on the point's own side of the field.
	const double S = (Ax + Y) / R2, C = (Ax - Y) / R2;
	// Home plate: 17 in across the front, its rear point at the origin.
	if (Y >= 0.0 && Y <= 2.0 * PlateHalf && Ax <= PlateHalf && Y >= Ax)
	{
		return Chalk;
	}
	bool bChalk = false;
	// Foul lines, in fair ground (the line is fair), from the batter's box to the pole.
	bChalk |= C <= 0.0 && C >= -ChalkW && Y > BoxFront && S <= PoleS[X < 0.0 ? 0 : 1];
	// Batter's boxes and the catcher's box.
	if (Y < BoxFront + 0.01 && Y > CatcherBack - 0.01 && Ax < BoxFar + 0.01)
	{
		const bool bBox = Ax >= BoxNear && Ax <= BoxFar && Y >= BoxBack && Y <= BoxFront;
		const bool bBoxInner = Ax >= BoxNear + BoxLineW && Ax <= BoxFar - BoxLineW && Y >= BoxBack + BoxLineW && Y <= BoxFront - BoxLineW;
		bChalk |= bBox && !bBoxInner;
		bChalk |= Y >= CatcherBack && Y <= BoxBack && Ax <= CatcherHalf && (Ax >= CatcherHalf - BoxLineW || Y <= CatcherBack + BoxLineW);
	}
	// The runner's lane on the first-base side.
	if (X > 0.0 && S >= LaneFrom && S <= BaseL && C > 0.0 && C <= PathHalf)
	{
		bChalk |= C >= PathHalf - BoxLineW || S <= LaneFrom + BoxLineW || S >= BaseL - BoxLineW;
	}
	// Coaches' boxes: a line 15 ft off each foul line with a short end at each end.
	if (S >= BaseL - CoachLen && S <= BaseL && C >= CoachOff && C <= CoachOff + CoachEnd)
	{
		bChalk |= C <= CoachOff + BoxLineW || S <= BaseL - CoachLen + BoxLineW || S >= BaseL - BoxLineW;
	}
	const FVector2D& Deck = OnDeck[X < 0.0 ? 0 : 1];
	const double DeckD2 = Dist2(X, Y, Deck);
	bChalk |= DeckD2 <= OnDeckR * OnDeckR && DeckD2 >= FMath::Square(OnDeckR - BoxLineW);
	if (bChalk)
	{
		return Chalk;
	}
	// The infield square's own axes: toward first and toward third from home.
	const double S1 = (X + Y) / R2, S3 = (Y - X) / R2;
	const bool bSquare = S1 >= 0.0 && S1 <= BaseL && S3 >= 0.0 && S3 <= BaseL;
	const FVector2D PlateC(0.0, PlateHalf);
	bool bDirt = DeckD2 < OnDeckR * OnDeckR || Dist2(X, Y, PlateC) < CutoutR * CutoutR || Dist2(X, Y, MoundC) < MoundR * MoundR;
	if (!bDirt && C < PathHalf && Dist2(X, Y, SkinC) < SkinR * SkinR)
	{
		// The infield skin: everything inside the arc and the lines except the infield
		// grass, which is the square less a base path's width and less the cut round each base.
		bool bGrass = S1 > PathHalf && S1 < BaseL - PathHalf && S3 > PathHalf && S3 < BaseL - PathHalf;
		for (int32 B = 0; bGrass && B < 3; ++B)
		{
			bGrass = Dist2(X, Y, FVector2D(Desc.bases[B][0], Desc.bases[B][1])) > CutoutR * CutoutR;
		}
		bDirt = !bGrass;
	}
	if (bDirt)
	{
		return Dirt;
	}
	if (FenceHere < TrackFence || SdHere < TrackWall)
	{
		return Track;
	}
	// Mowing (owner, 2026-11-15): the infield grass is one shade; everything outside the
	// square is cut in plain stripes running from home toward centre field, one centred on
	// the line through the plate and second base.
	if (bSquare)
	{
		return GrassLight;
	}
	return (FMath::FloorToInt32(X / Mow + 0.5) & 1) ? GrassDark : GrassLight;
}

ABbPark::ABbPark()
{
	RootComponent = CreateDefaultSubobject<USceneComponent>(TEXT("Root"));
}

AStaticMeshActor* ABbPark::Shape(UStaticMesh* Mesh, const FVector& Location, const FVector& Scale, const FLinearColor& Color, const FRotator& Rotation)
{
	AStaticMeshActor* A = GetWorld()->SpawnActor<AStaticMeshActor>(Location, Rotation);
	UStaticMeshComponent* C = A->GetStaticMeshComponent();
	C->SetMobility(EComponentMobility::Movable);
	C->SetStaticMesh(Mesh);
	C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
	A->SetActorScale3D(Scale);
	if (FlatMaterial)
	{
		UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(FlatMaterial, A);
		M->SetVectorParameterValue(TEXT("Color"), Color);
		C->SetMaterial(0, M);
	}
	return A;
}

AStaticMeshActor* ABbPark::Panel(const FVector2D& A, const FVector2D& B, double Depth, double Z0, double Z1, const FLinearColor& Color)
{
	// Engine primitives are 100 cm across. The cube's X is turned to run from A to B.
	const FVector2D Mid = (A + B) * 0.5;
	const FVector Dir = ToUE(B.X - A.X, B.Y - A.Y, 0).GetSafeNormal();
	return Shape(Cube, ToUE(Mid.X, Mid.Y, (Z0 + Z1) * 0.5), FVector((B - A).Size() + 0.05, Depth, Z1 - Z0), Color, Dir.Rotation());
}

void ABbPark::BuildLights()
{
	UWorld* W = GetWorld();
	// The template map may bring its own sky and sun; only add what is missing.
	TArray<AActor*> Found;
	// No distance haze: the pitch camera looks down 140 m of field through a long lens.
	UGameplayStatics::GetAllActorsOfClass(W, AExponentialHeightFog::StaticClass(), Found);
	for (AActor* A : Found)
	{
		A->Destroy();
	}
	Found.Reset();
	UGameplayStatics::GetAllActorsOfClass(W, ADirectionalLight::StaticClass(), Found);
	// An early-afternoon sun over the third-base side lights both camera angles.
	for (AActor* A : Found)
	{
		A->SetActorRotation(FRotator(-55.f, 100.f, 0.f));
	}
	if (Found.Num() == 0)
	{
		ADirectionalLight* Sun = W->SpawnActor<ADirectionalLight>(FVector(0, 0, 5000), FRotator(-55.f, 100.f, 0.f));
		Sun->GetLightComponent()->SetMobility(EComponentMobility::Movable);
		Sun->GetLightComponent()->SetIntensity(6.f);
		if (UDirectionalLightComponent* D = Cast<UDirectionalLightComponent>(Sun->GetLightComponent()))
		{
			D->SetAtmosphereSunLight(true);
			D->DynamicShadowDistanceMovableLight = 30000.f;
		}
	}
	Found.Reset();
	UGameplayStatics::GetAllActorsOfClass(W, ASkyLight::StaticClass(), Found);
	if (Found.Num() == 0)
	{
		AActor* Atmosphere = W->SpawnActor<AActor>(AActor::StaticClass());
		USkyAtmosphereComponent* Sky = NewObject<USkyAtmosphereComponent>(Atmosphere);
		Atmosphere->SetRootComponent(Sky);
		Sky->RegisterComponent();
		ASkyLight* SkyLight = W->SpawnActor<ASkyLight>();
		SkyLight->GetLightComponent()->SetMobility(EComponentMobility::Movable);
		SkyLight->GetLightComponent()->bRealTimeCapture = true;
		SkyLight->GetLightComponent()->SetIntensity(1.2f);
		SkyLight->GetLightComponent()->RecaptureSky();
	}
}

void ABbPark::Build(const BbPark& Desc)
{
	const double Began = FPlatformTime::Seconds();
	Cube = LoadObject<UStaticMesh>(nullptr, TEXT("/Engine/BasicShapes/Cube.Cube"));
	Cylinder = LoadObject<UStaticMesh>(nullptr, TEXT("/Engine/BasicShapes/Cylinder.Cylinder"));
	Plane = LoadObject<UStaticMesh>(nullptr, TEXT("/Engine/BasicShapes/Plane.Plane"));
	FlatMaterial = LoadObject<UMaterialInterface>(nullptr, TEXT("/Engine/BasicShapes/BasicShapeMaterial.BasicShapeMaterial"));
#if WITH_EDITOR
	{
		// The engine's shape material takes one flat colour. The ground needs a picture, so a
		// lit material with a texture parameter is put together here (park-visuals §2): in
		// code, nothing saved, no asset in the repository.
		UMaterial* M = NewObject<UMaterial>(GetTransientPackage(), NAME_None, RF_Transient);
		UMaterialExpressionTextureSampleParameter2D* Tex = NewObject<UMaterialExpressionTextureSampleParameter2D>(M);
		Tex->ParameterName = TEXT("Tex");
		Tex->Texture = LoadObject<UTexture2D>(nullptr, TEXT("/Engine/EngineResources/WhiteSquareTexture.WhiteSquareTexture"));
		Tex->SamplerType = SAMPLERTYPE_Color;
		UMaterialExpressionConstant* Rough = NewObject<UMaterialExpressionConstant>(M);
		Rough->R = 0.95f;
		UMaterialExpressionConstant* Spec = NewObject<UMaterialExpressionConstant>(M);
		Spec->R = 0.05f;
		M->GetExpressionCollection().AddExpression(Tex);
		M->GetExpressionCollection().AddExpression(Rough);
		M->GetExpressionCollection().AddExpression(Spec);
		M->GetEditorOnlyData()->BaseColor.Connect(0, Tex);
		M->GetEditorOnlyData()->Roughness.Connect(0, Rough);
		M->GetEditorOnlyData()->Specular.Connect(0, Spec);
		M->PostEditChange();
		TexturedMaterial = M;
	}
#endif
	Ground = MakeUnique<FBbGround>(Desc);
	BuildSurface();
	BuildFurniture();
	BuildWalls();
	BuildStands();
	UE_LOG(LogBbPark, Display, TEXT("BB_PARK built in %.2f s: %u wall pieces, ground %.0f m square from (%.0f, %.0f)"),
		FPlatformTime::Seconds() - Began, Desc.n_wall, Ground->Size, Ground->X0, Ground->Y0);
}

UTexture2D* ABbPark::PaintGround(double TexX0, double TexY0, double TexSize, int32 N)
{
	// Each texel is the mean of four points in it, taken in linear light, so edges are smooth.
	TArray<FColor> Top;
	Top.SetNumUninitialized(N * N);
	const FBbGround& G = *Ground;
	ParallelFor(N, [&](int32 J)
	{
		for (int32 I = 0; I < N; ++I)
		{
			FLinearColor Sum(0, 0, 0, 0);
			for (int32 K = 0; K < 4; ++K)
			{
				Sum += G.Color(TexX0 + (I + 0.25 + 0.5 * (K & 1)) * TexSize / N, TexY0 + (J + 0.25 + 0.5 * (K >> 1)) * TexSize / N);
			}
			Sum *= 0.25f;
			Sum.A = 1.f;
			Top[J * N + I] = Sum.ToFColor(true);
		}
	});
	UTexture2D* T = UTexture2D::CreateTransient(N, N, PF_B8G8R8A8);
	T->SRGB = true;
	T->NeverStream = true;
	T->LODGroup = TEXTUREGROUP_World;
	T->AddressX = TA_Clamp;
	T->AddressY = TA_Clamp;
	FTexturePlatformData* Data = T->GetPlatformData();
	{
		void* P = Data->Mips[0].BulkData.Lock(LOCK_READ_WRITE);
		FMemory::Memcpy(P, Top.GetData(), (SIZE_T)N * N * 4);
		Data->Mips[0].BulkData.Unlock();
	}
	// The smaller copies a far or slanting view reads from, each the mean of four texels of
	// the one before, again in linear light: without them the chalk and the mowing shimmer.
	TArray<FColor> Cur = MoveTemp(Top);
	for (int32 W = N / 2; W >= 1; W /= 2)
	{
		TArray<FColor> Next;
		Next.SetNumUninitialized(W * W);
		ParallelFor(W, [&](int32 J)
		{
			for (int32 I = 0; I < W; ++I)
			{
				FLinearColor Sum(0, 0, 0, 0);
				for (int32 K = 0; K < 4; ++K)
				{
					Sum += FLinearColor(Cur[(2 * J + (K >> 1)) * 2 * W + 2 * I + (K & 1)]);
				}
				Sum *= 0.25f;
				Sum.A = 1.f;
				Next[J * W + I] = Sum.ToFColor(true);
			}
		});
		FTexture2DMipMap* Mip = new FTexture2DMipMap(W, W, 1);
		Data->Mips.Add(Mip);
		Mip->BulkData.Lock(LOCK_READ_WRITE);
		void* P = Mip->BulkData.Realloc((int64)W * W * 4);
		FMemory::Memcpy(P, Next.GetData(), (SIZE_T)W * W * 4);
		Mip->BulkData.Unlock();
		Cur = MoveTemp(Next);
	}
	T->UpdateResource();
	Textures.Add(T);
	return T;
}

void ABbPark::AddGround(int32 Section, double GX0, double GY0, double Cell, int32 Nx, int32 Ny, const FBox2D& Hole, UTexture2D* Tex, double TexX0, double TexY0, double TexSize)
{
	TArray<FVector> Verts, Normals;
	TArray<FVector2D> UVs;
	TArray<int32> Tris;
	TArray<FProcMeshTangent> Tangents;
	const int32 Row = Nx + 1;
	Verts.Reserve(Row * (Ny + 1));
	const FBbGround& G = *Ground;
	for (int32 J = 0; J <= Ny; ++J)
	{
		for (int32 I = 0; I <= Nx; ++I)
		{
			const double X = GX0 + I * Cell, Y = GY0 + J * Cell;
			Verts.Add(ToUE(X, Y, G.Height(X, Y)));
			// The surface's normal from its slope (sim frame), then into Unreal's frame.
			const double E = 0.02;
			const double Sx = (G.Height(X + E, Y) - G.Height(X - E, Y)) / (2 * E), Sy = (G.Height(X, Y + E) - G.Height(X, Y - E)) / (2 * E);
			Normals.Add(FVector(-Sy, -Sx, 1.0).GetSafeNormal());
			UVs.Add(FVector2D((X - TexX0) / TexSize, (Y - TexY0) / TexSize));
			Tangents.Add(FProcMeshTangent(0.f, 1.f, 0.f));
		}
	}
	for (int32 J = 0; J < Ny; ++J)
	{
		for (int32 I = 0; I < Nx; ++I)
		{
			const FVector2D Mid(GX0 + (I + 0.5) * Cell, GY0 + (J + 0.5) * Cell);
			if (Hole.bIsValid && Hole.IsInside(Mid))
			{
				continue;
			}
			// Facing up in Unreal's left-handed frame: the corner, then sim +X, then sim +Y.
			const int32 K = J * Row + I;
			Tris.Append({ K, K + 1, K + Row, K + Row, K + 1, K + Row + 1 });
		}
	}
	Surface->CreateMeshSection_LinearColor(Section, Verts, Tris, Normals, UVs, TArray<FLinearColor>(), Tangents, false);
	if (TexturedMaterial && Tex)
	{
		UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(TexturedMaterial, this);
		M->SetTextureParameterValue(TEXT("Tex"), Tex);
		Surface->SetMaterial(Section, M);
	}
	else if (FlatMaterial)
	{
		// No textured material (a build without the editor's material compiler): plain grass.
		UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(FlatMaterial, this);
		M->SetVectorParameterValue(TEXT("Color"), GrassLight);
		Surface->SetMaterial(Section, M);
	}
}

void ABbPark::BuildSurface()
{
	const FBbGround& G = *Ground;
	Surface = NewObject<UProceduralMeshComponent>(this);
	Surface->SetupAttachment(RootComponent);
	Surface->SetMobility(EComponentMobility::Movable);
	Surface->SetCollisionEnabled(ECollisionEnabled::NoCollision);
	Surface->SetCastShadow(false);
	Surface->RegisterComponent();
	// Two pictures of the ground: the whole park at about 4 cm a texel, and the infield at
	// 1.4 cm, where the boxes and the plate are.
	const int32 N = TexturedMaterial ? 4096 : 0;
	UTexture2D* Far = N ? PaintGround(G.X0, G.Y0, G.Size, N) : nullptr;
	UTexture2D* Near = N ? PaintGround(NearX0, NearY0, NearSize, N) : nullptr;
	// The infield in 0.25 m cells, so the mound has its shape; the rest, which is level, in
	// 2 m cells with the infield's square left out.
	const FBox2D NearBox(FVector2D(NearX0, NearY0), FVector2D(NearX0 + NearSize, NearY0 + NearSize));
	AddGround(0, NearX0, NearY0, 0.25, FMath::RoundToInt32(NearSize / 0.25), FMath::RoundToInt32(NearSize / 0.25), FBox2D(ForceInit), Near, NearX0, NearY0, NearSize);
	AddGround(1, G.X0, G.Y0, 2.0, FMath::RoundToInt32(G.Size / 2.0), FMath::RoundToInt32(G.Size / 2.0), NearBox, Far, G.X0, G.Y0, G.Size);
	// The land the park stands on, a step below the field.
	Shape(Plane, ToUE(G.X0 + G.Size * 0.5, G.Y0 + G.Size * 0.5, -0.3), FVector(6.0 * G.Size, 6.0 * G.Size, 1), GrassDark)->GetStaticMeshComponent()->SetCastShadow(false);
}

void ABbPark::BuildFurniture()
{
	const BbPark& D = Ground->Desc;
	// Bases: 18 in square and 3 in proud. First and third lie inside the infield square, in
	// its corner; second is centred on its corner.
	const FVector2D B1(D.bases[0][0], D.bases[0][1]), B2(D.bases[1][0], D.bases[1][1]), B3(D.bases[2][0], D.bases[2][1]);
	const FVector2D In1 = (-B1).GetSafeNormal() + (B2 - B1).GetSafeNormal(), In3 = (-B3).GetSafeNormal() + (B2 - B3).GetSafeNormal();
	const FVector2D At[3] = { B1 + In1 * (BaseSide * 0.5), B2, B3 + In3 * (BaseSide * 0.5) };
	for (const FVector2D& P : At)
	{
		Shape(Cube, ToUE(P.X, P.Y, 0.038), FVector(BaseSide, BaseSide, 0.076), Chalk, FRotator(0, 45, 0));
	}
	// The rubber: 24 by 6 in, its front edge where the sim has it, on the mound's table.
	const double RubY = D.rubber_m + 0.25 * FT;
	Shape(Cube, ToUE(0, RubY, Ground->Height(0, RubY) + 0.005), FVector(0.5 * FT, 2.0 * FT, 0.03), Chalk);
	// Foul poles.
	for (int32 I = 0; I < 2; ++I)
	{
		Shape(Cylinder, ToUE(D.poles[I][0], D.poles[I][1], 6.0), FVector(0.3, 0.3, 12.0), FLinearColor(0.9f, 0.8f, 0.1f));
	}
}

void ABbPark::BuildWalls()
{
	// Every wall stands where the sim has it (physics-spec §9.5), as high as the sim has it.
	const BbPark& D = Ground->Desc;
	const FLinearColor Shade(0.03f, 0.03f, 0.035f);
	const int32 N = D.n_wall;
	for (int32 I = 0; I < N; ++I)
	{
		const BbWallPiece& P = D.wall[I];
		const FVector2D A(P.from[0], P.from[1]), B(D.wall[(I + 1) % N].from[0], D.wall[(I + 1) % N].from[1]);
		// Out of the field: to the left of the piece's direction.
		const FVector2D Along = (B - A).GetSafeNormal(), Out(-Along.Y, Along.X);
		if (P.kind == 4)
		{
			// A dugout: a dark bay behind the opening, under a roof.
			const double Deep = 3.5;
			Panel(A + Out * (Deep * 0.5), B + Out * (Deep * 0.5), Deep, -0.01, 0.01, Shade)->GetStaticMeshComponent()->SetCastShadow(false);
			Panel(A + Out * (Deep * 0.5 + 0.3), B + Out * (Deep * 0.5 + 0.3), Deep - 0.6, 2.3, 2.5, WallColor);
		}
		else if (P.kind != 0 && P.top_m > D.foul_wall_height_m + 0.01f)
		{
			// Behind home the walls carry the screen: a low wall, and the screen drawn as its
			// posts and top rail only (the net itself is see-through).
			Panel(A, B, 0.3, 0.0, D.foul_wall_height_m, WallColor);
			for (const FVector2D& Q : { A, B })
			{
				Shape(Cylinder, ToUE(Q.X, Q.Y, P.top_m * 0.5), FVector(0.12, 0.12, P.top_m), Chalk);
			}
			Panel(A, B, 0.08, P.top_m - 0.04, P.top_m + 0.04, Chalk);
		}
		else
		{
			Panel(A, B, 0.3, 0.0, P.top_m, WallColor);
		}
	}
}

void ABbPark::BuildStands()
{
	// Seating as plain blocks, so the park reads as a park (the seating bowl proper is
	// park-visuals step 4): two tiers stepping up and back from every wall. None of it is in
	// the sim.
	const BbPark& D = Ground->Desc;
	const FLinearColor Seats(0.2f, 0.22f, 0.28f);
	const FLinearColor ConcreteTier(0.45f, 0.45f, 0.47f);
	const int32 N = D.n_wall;
	for (int32 I = 0; I < N; ++I)
	{
		const BbWallPiece& P = D.wall[I];
		const FVector2D A(P.from[0], P.from[1]), B(D.wall[(I + 1) % N].from[0], D.wall[(I + 1) % N].from[1]);
		const FVector2D Along = (B - A).GetSafeNormal(), Out(-Along.Y, Along.X);
		const bool bFence = P.kind == 0;
		for (int32 Tier = 0; Tier < 2; ++Tier)
		{
			// Beyond the fence the bank starts close; in foul ground it starts behind the
			// dugouts' roofs.
			const double Back = bFence ? 5.0 + 7.0 * Tier : 8.0 + 8.0 * Tier;
			const double Depth = bFence ? 7.0 : 8.0;
			const double High = bFence ? 5.0 + 4.5 * Tier : 5.0 + 5.0 * Tier;
			// Each block reaches a little past its piece's ends, so the tiers close up round
			// the outside of a corner.
			const FVector2D Reach = Along * (Back * 0.42);
			Panel(A + Out * Back - Reach, B + Out * Back + Reach, Depth, 0.0, High, Tier == 0 ? Seats : ConcreteTier)->GetStaticMeshComponent()->SetCastShadow(false);
		}
	}
}

FVector2D ABbPark::DugoutDoor(double Side) const
{
	if (Ground)
	{
		const BbPark& D = Ground->Desc;
		const int32 N = D.n_wall;
		for (int32 I = 0; I < N; ++I)
		{
			const FVector2D A(D.wall[I].from[0], D.wall[I].from[1]), B(D.wall[(I + 1) % N].from[0], D.wall[(I + 1) % N].from[1]);
			if (D.wall[I].kind == 4 && (A.X + B.X) * Side > 0.0)
			{
				const FVector2D Along = (B - A).GetSafeNormal();
				return (A + B) * 0.5 + FVector2D(-Along.Y, Along.X) * 1.5;
			}
		}
	}
	return FVector2D(Side * 20.0, -2.0);
}

TArray<FBbParkSpot> ABbPark::Spots(const BbPark& Desc)
{
	// The park's reach from home: to the deepest fence and to the widest point.
	double Deep = 0.0, Wide = 0.0;
	for (uint32 I = 0; I < Desc.n_wall; ++I)
	{
		Deep = FMath::Max(Deep, (double)Desc.wall[I].from[1]);
		Wide = FMath::Max(Wide, FMath::Abs((double)Desc.wall[I].from[0]));
	}
	auto At = [](const TCHAR* Name, const FVector& Eye, const FVector& Target, float Fov)
	{
		return FBbParkSpot{ Name, Eye, (Target - Eye).Rotation(), Fov };
	};
	TArray<FBbParkSpot> S;
	S.Add(At(TEXT("home"), ToUE(0, -30, 16), ToUE(0, 35, 0), 60.f));
	S.Add(At(TEXT("center"), ToUE(0, Deep + 8, 14), ToUE(0, 20, 0), 40.f));
	S.Add(At(TEXT("first_high"), ToUE(Wide, -5, 38), ToUE(0, 42, 0), 60.f));
	S.Add(At(TEXT("third_stands"), ToUE(-45, -15, 18), ToUE(5, 30, 0), 65.f));
	S.Add(At(TEXT("mound"), ToUE(4.5, Desc.rubber_m - 7.0, 0.9), ToUE(0, Desc.rubber_m - 0.3, 0.35), 40.f));
	// Straight down, centre field at the top of the picture, the whole park in a 16:9 frame.
	const double MidY = (Deep - 20.0) * 0.5;
	const double Half = FMath::Max((Deep + 30.0) * 0.5, Wide * 9.0 / 16.0 + 8.0);
	const float PlanFov = 50.f;
	const double High = Half / (FMath::Tan(FMath::DegreesToRadians(PlanFov * 0.5)) * 9.0 / 16.0);
	S.Add(FBbParkSpot{ TEXT("plan"), ToUE(0, MidY, High), FRotator(-90.f, 0.f, 0.f), PlanFov });
	return S;
}
