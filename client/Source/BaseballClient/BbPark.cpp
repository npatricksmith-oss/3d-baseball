#include "BbPark.h"
#include "BbFrame.h"
#include "Algo/Reverse.h"
#include "Async/ParallelFor.h"
#include "BbLook.h"
#include "Components/DirectionalLightComponent.h"
#include "Components/InstancedStaticMeshComponent.h"
#include "Components/PostProcessComponent.h"
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
#include "Materials/MaterialExpressionScalarParameter.h"
#include "Materials/MaterialExpressionVectorParameter.h"
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

	// The ground's colours come from the park's one palette (BbLook.h).
	using BbLook::GrassLight;
	using BbLook::GrassDark;
	using BbLook::Dirt;
	using BbLook::Track;
	using BbLook::Chalk;

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
	// A dugout (none of it in the sim but the opening and the roof's height): three steps down
	// from the field to a floor the roof clears by two metres.
	const double PitDeep = 3.2;               // from the opening back to the rear wall
	const double StepRun = 0.4, StepRise = 0.3;
	const int32 PitSteps = 3;
	const double PitFloor = -StepRise * PitSteps;
	const double PitWall = 0.5;               // its walls, and the lip along the field's edge
	const double RoofThick = 0.12;
	// The field's walls are this thick, their faces where the sim has the boundary.
	const double WallThick = 0.3;
	// Behind the fence: the batter's eye either side of dead centre, and a bullpen in each
	// alley, by the fence's bearing from home (degrees from centre field).
	const double EyeHalfDeg = 8.0, EyeHigh = 11.0, EyeBack = 1.5;
	const double PenFromDeg = EyeHalfDeg, PenToDeg = 22.0;
	const double PenDeep = 7.0;               // from the fence back to the pen's own wall
	const double PenLane[2] = { 2.0, 5.0 };   // the two mounds, this far behind the fence
	const double PenPitch = 60.5 * FT, PenMoundR = 2.2, PenPlateR = 1.6;
	// Foul poles and their screens.
	const double PoleHigh = 12.0, PoleScreen = 0.8;
	// The distance figures on the fence.
	const double MarkW = 2.4, MarkH = 1.3, MarkZ = 1.2;
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

	// The dugouts' openings, and what stands behind each piece of the fence.
	FMemory::Memzero(Behind);
	TArray<FBbRun> PenSide[2];
	for (uint32 I = 0; I < Desc.n_wall; ++I)
	{
		const BbWallPiece& P = Desc.wall[I];
		const BbWallPiece& Q = Desc.wall[(I + 1) % Desc.n_wall];
		const FVector2D A(P.from[0], P.from[1]), B(Q.from[0], Q.from[1]);
		const FVector2D Along = (B - A).GetSafeNormal();
		const FBbRun Run{ A, Along, FVector2D(-Along.Y, Along.X), (B - A).Size() };
		if (P.kind == 4)
		{
			Dugouts.Add(Run);
		}
		else if (P.kind == 0)
		{
			const FVector2D Mid = (A + B) * 0.5;
			const double Deg = FMath::RadiansToDegrees(FMath::Atan2(Mid.X, Mid.Y));
			if (FMath::Abs(Deg) < EyeHalfDeg)
			{
				Behind[I] = Eye;
			}
			else if (FMath::Abs(Deg) >= PenFromDeg && FMath::Abs(Deg) <= PenToDeg)
			{
				Behind[I] = Pen;
				PenSide[Deg < 0.0 ? 0 : 1].Add(Run);
			}
		}
	}
	for (int32 Side = 0; Side < 2; ++Side)
	{
		// Along the fence from the pen's end nearer centre field. The loop runs from the
		// left-field pole to the right, so the left pen's pieces are turned round.
		TArray<FBbRun> Out = PenSide[Side];
		if (Side == 0)
		{
			Algo::Reverse(Out);
			for (FBbRun& R : Out)
			{
				R.A += R.Along * R.L;
				R.Along = -R.Along;
			}
		}
		double Total = 0.0;
		for (const FBbRun& R : Out)
		{
			Total += R.L;
		}
		// Room for the pitch and three metres behind each end, or no pen.
		if (Total < PenPitch + 6.0)
		{
			for (uint32 I = 0; I < Desc.n_wall; ++I)
			{
				const bool bThisSide = (Desc.wall[I].from[0] + Desc.wall[(I + 1) % Desc.n_wall].from[0] < 0.f) == (Side == 0);
				if (Behind[I] == Pen && bThisSide)
				{
					Behind[I] = Plain;
				}
			}
			continue;
		}
		auto At = [&Out](double S, double Depth)
		{
			for (const FBbRun& R : Out)
			{
				if (S <= R.L)
				{
					return R.A + R.Along * S + R.Out * Depth;
				}
				S -= R.L;
			}
			return Out.Last().A + Out.Last().Along * Out.Last().L + Out.Last().Out * Depth;
		};
		for (double Lane : PenLane)
		{
			PenMounds.Add(At((Total - PenPitch) * 0.5, Lane));
			PenPlates.Add(At((Total + PenPitch) * 0.5, Lane));
		}
		Pens.Append(PenSide[Side]);
	}

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
		// Behind a dugout's opening: the steps down to its floor.
		for (const FBbRun& R : Dugouts)
		{
			const FVector2D P(X - R.A.X, Y - R.A.Y);
			const double A = P | R.Along, O = P | R.Out;
			if (A >= 0.0 && A <= R.L && O > 0.0 && O <= PitDeep)
			{
				return -StepRise * FMath::Min(PitSteps, 1 + FMath::FloorToInt32(O / StepRun));
			}
		}
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
		// A bullpen behind the fence: grass, with dirt round its mounds and plates.
		if (FenceHere < PenDeep + 0.5f)
		{
			for (const FBbRun& R : Pens)
			{
				const FVector2D P(X - R.A.X, Y - R.A.Y);
				const double A = P | R.Along, O = P | R.Out;
				if (A < -0.15 || A > R.L + 0.15 || O <= 0.0 || O > PenDeep)
				{
					continue;
				}
				for (const FVector2D& M : PenMounds)
				{
					if (Dist2(X, Y, M) < PenMoundR * PenMoundR)
					{
						return Dirt;
					}
				}
				for (const FVector2D& H : PenPlates)
				{
					if (Dist2(X, Y, H) < PenPlateR * PenPlateR)
					{
						return Dirt;
					}
				}
				return GrassLight;
			}
		}
		return BbLook::Concrete.Color;
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

AStaticMeshActor* ABbPark::Shape(UStaticMesh* Mesh, const FVector& Location, const FVector& Scale, const FBbPaint& Paint, const FRotator& Rotation)
{
	AStaticMeshActor* A = GetWorld()->SpawnActor<AStaticMeshActor>(Location, Rotation);
	UStaticMeshComponent* C = A->GetStaticMeshComponent();
	C->SetMobility(EComponentMobility::Movable);
	C->SetStaticMesh(Mesh);
	C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
	A->SetActorScale3D(Scale);
	if (UMaterialInterface* Base = PaintMaterial ? PaintMaterial.Get() : FlatMaterial.Get())
	{
		UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(Base, A);
		M->SetVectorParameterValue(TEXT("Color"), Paint.Color);
		M->SetScalarParameterValue(TEXT("Rough"), Paint.Rough);
		C->SetMaterial(0, M);
	}
	return A;
}

void ABbPark::Panel(const FVector2D& A, const FVector2D& B, double Depth, double Z0, double Z1, const FBbPaint& Paint, bool bShadow)
{
	// Engine primitives are 100 cm across. The cube's X is turned to run from A to B.
	const FVector2D Mid = (A + B) * 0.5;
	const FVector Dir = ToUE(B.X - A.X, B.Y - A.Y, 0).GetSafeNormal();
	Inst(Cube, ToUE(Mid.X, Mid.Y, (Z0 + Z1) * 0.5), FVector((B - A).Size() + 0.05, Depth, Z1 - Z0), Paint, Dir.Rotation(), bShadow);
}

void ABbPark::BuildLights()
{
	UWorld* W = GetWorld();
	// The template map may bring its own sky and sun. Whatever is there is set to the park's
	// light (BbLook.h); what is missing is added.
	TArray<AActor*> Found;
	// No distance haze: the pitch camera looks down 140 m of field through a long lens.
	UGameplayStatics::GetAllActorsOfClass(W, AExponentialHeightFog::StaticClass(), Found);
	for (AActor* A : Found)
	{
		A->Destroy();
	}
	// The sun's rays run from where it stands down to the field.
	const double Bearing = FMath::DegreesToRadians(BbLook::SunBearingDeg), Height = FMath::DegreesToRadians(BbLook::SunHeightDeg);
	const FRotator Rays = (-ToUE(FMath::Sin(Bearing) * FMath::Cos(Height), FMath::Cos(Bearing) * FMath::Cos(Height), FMath::Sin(Height))).Rotation();
	Found.Reset();
	UGameplayStatics::GetAllActorsOfClass(W, ADirectionalLight::StaticClass(), Found);
	if (Found.Num() == 0)
	{
		Found.Add(W->SpawnActor<ADirectionalLight>(FVector(0, 0, 5000), Rays));
	}
	for (AActor* A : Found)
	{
		UDirectionalLightComponent* D = Cast<UDirectionalLightComponent>(CastChecked<ADirectionalLight>(A)->GetLightComponent());
		if (!D)
		{
			continue;
		}
		D->SetMobility(EComponentMobility::Movable);
		A->SetActorRotation(Rays);
		D->SetIntensity(BbLook::SunLux);
		D->SetUseTemperature(false);
		D->SetLightColor(BbLook::SunColor);
		D->SetAtmosphereSunLight(true);
		D->SetDynamicShadowDistanceMovableLight(BbLook::ShadowReachM * 100.f);
		D->SetDynamicShadowCascades(BbLook::ShadowCascades);
		D->SetCascadeDistributionExponent(BbLook::ShadowCascadeExponent);
		D->SetCascadeTransitionFraction(0.15f);
		D->SetShadowDistanceFadeoutFraction(0.05f);
	}
	Found.Reset();
	UGameplayStatics::GetAllActorsOfClass(W, ASkyLight::StaticClass(), Found);
	if (Found.Num() == 0)
	{
		AActor* Atmosphere = W->SpawnActor<AActor>(AActor::StaticClass());
		USkyAtmosphereComponent* Sky = NewObject<USkyAtmosphereComponent>(Atmosphere);
		Atmosphere->SetRootComponent(Sky);
		Sky->RegisterComponent();
		Found.Add(W->SpawnActor<ASkyLight>());
	}
	for (AActor* A : Found)
	{
		USkyLightComponent* S = CastChecked<ASkyLight>(A)->GetLightComponent();
		S->SetMobility(EComponentMobility::Movable);
		S->bRealTimeCapture = true;
		S->SetIntensity(BbLook::SkyFill);
		// From below, the light that comes back up off the grass rather than a black floor.
		S->bLowerHemisphereIsBlack = true;
		S->SetLowerHemisphereColor(BbLook::GroundBounce);
		S->MarkRenderStateDirty();
		S->RecaptureSky();
	}
	// The picture, for every camera: fixed exposure, a little shade where surfaces meet, and
	// a light grade.
	UPostProcessComponent* P = NewObject<UPostProcessComponent>(this);
	P->SetupAttachment(RootComponent);
	P->bUnbound = true;
	FPostProcessSettings& S = P->Settings;
	S.bOverride_AutoExposureBias = true;
	S.AutoExposureBias = BbLook::ExposureStops;
	S.bOverride_AmbientOcclusionIntensity = true;
	S.AmbientOcclusionIntensity = BbLook::AoIntensity;
	S.bOverride_AmbientOcclusionRadius = true;
	S.AmbientOcclusionRadius = BbLook::AoRadiusCm;
	S.bOverride_AmbientOcclusionQuality = true;
	S.AmbientOcclusionQuality = 70.f;
	S.bOverride_ColorSaturation = true;
	S.ColorSaturation = FVector4(BbLook::Saturation, BbLook::Saturation, BbLook::Saturation, 1.0);
	S.bOverride_ColorContrast = true;
	S.ColorContrast = FVector4(BbLook::Contrast, BbLook::Contrast, BbLook::Contrast, 1.0);
	S.bOverride_BloomIntensity = true;
	S.BloomIntensity = BbLook::BloomIntensity;
	S.bOverride_VignetteIntensity = true;
	S.VignetteIntensity = BbLook::Vignette;
	P->RegisterComponent();
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
		// The engine's shape material takes one flat colour and nothing else. The park needs
		// two of its own (park-visuals §2), put together here in code, nothing saved, no asset
		// in the repository: paint (a colour and how matt it is) and the ground (a picture).
		auto Finish = [](UMaterial* M, UMaterialExpression* Color, UMaterialExpression* Rough)
		{
			UMaterialExpressionConstant* Spec = NewObject<UMaterialExpressionConstant>(M);
			Spec->R = 0.3f;
			for (UMaterialExpression* E : { Color, Rough, (UMaterialExpression*)Spec })
			{
				M->GetExpressionCollection().AddExpression(E);
			}
			M->GetEditorOnlyData()->BaseColor.Connect(0, Color);
			M->GetEditorOnlyData()->Roughness.Connect(0, Rough);
			M->GetEditorOnlyData()->Specular.Connect(0, Spec);
			M->bUsedWithInstancedStaticMeshes = true;
			M->PostEditChange();
		};
		{
			UMaterial* M = NewObject<UMaterial>(GetTransientPackage(), NAME_None, RF_Transient);
			UMaterialExpressionVectorParameter* Color = NewObject<UMaterialExpressionVectorParameter>(M);
			Color->ParameterName = TEXT("Color");
			Color->DefaultValue = FLinearColor(0.5f, 0.5f, 0.5f);
			UMaterialExpressionScalarParameter* Rough = NewObject<UMaterialExpressionScalarParameter>(M);
			Rough->ParameterName = TEXT("Rough");
			Rough->DefaultValue = 0.9f;
			Finish(M, Color, Rough);
			PaintMaterial = M;
		}
		{
			UMaterial* M = NewObject<UMaterial>(GetTransientPackage(), NAME_None, RF_Transient);
			UMaterialExpressionTextureSampleParameter2D* Tex = NewObject<UMaterialExpressionTextureSampleParameter2D>(M);
			Tex->ParameterName = TEXT("Tex");
			Tex->Texture = LoadObject<UTexture2D>(nullptr, TEXT("/Engine/EngineResources/WhiteSquareTexture.WhiteSquareTexture"));
			Tex->SamplerType = SAMPLERTYPE_Color;
			UMaterialExpressionScalarParameter* Rough = NewObject<UMaterialExpressionScalarParameter>(M);
			Rough->ParameterName = TEXT("Rough");
			Rough->DefaultValue = BbLook::GroundRough;
			Finish(M, Tex, Rough);
			TexturedMaterial = M;
		}
		{
			// The net: unlit and see-through, so it only greys a little what is behind it.
			UMaterial* M = NewObject<UMaterial>(GetTransientPackage(), NAME_None, RF_Transient);
			M->BlendMode = BLEND_Translucent;
			M->SetShadingModel(MSM_Unlit);
			UMaterialExpressionVectorParameter* Color = NewObject<UMaterialExpressionVectorParameter>(M);
			Color->ParameterName = TEXT("Color");
			Color->DefaultValue = BbLook::Net;
			UMaterialExpressionScalarParameter* Opacity = NewObject<UMaterialExpressionScalarParameter>(M);
			Opacity->ParameterName = TEXT("Opacity");
			Opacity->DefaultValue = BbLook::NetOpacity;
			M->GetExpressionCollection().AddExpression(Color);
			M->GetExpressionCollection().AddExpression(Opacity);
			M->GetEditorOnlyData()->EmissiveColor.Connect(0, Color);
			M->GetEditorOnlyData()->Opacity.Connect(0, Opacity);
			M->PostEditChange();
			NetMaterial = M;
		}
	}
#endif
	Ground = MakeUnique<FBbGround>(Desc);
	BuildSurface();
	BuildFurniture();
	BuildWalls();
	BuildDugouts();
	BuildOutfield();
	BuildStands();
	FlushInst();
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
	return MakeTexture(MoveTemp(Top), N);
}

UTexture2D* ABbPark::MakeTexture(TArray<FColor>&& Top, int32 N)
{
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

void ABbPark::AddGround(int32 Section, double GX0, double GY0, double Cell, int32 Nx, int32 Ny, TFunctionRef<bool(const FVector2D&)> Skip, UTexture2D* Tex, double TexX0, double TexY0, double TexSize)
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
			if (Skip(Mid))
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
	// The ground is left open over each dugout, a little wider than the pit so that no cell
	// reaches into it; the dugout's walls and the lip along the field's edge cover the rest.
	// Round each pit the ground is in fine cells, so the opening keeps its shape.
	auto InPit = [&G](const FVector2D& P)
	{
		for (const FBbRun& R : G.Dugouts)
		{
			const double A = (P - R.A) | R.Along, O = (P - R.A) | R.Out;
			if (A > -0.25 && A < R.L + 0.25 && O > -0.25 && O < PitDeep + 0.25)
			{
				return true;
			}
		}
		return false;
	};
	TArray<FBox2D> PitBoxes;
	for (const FBbRun& R : G.Dugouts)
	{
		FBox2D Box(ForceInit);
		for (int32 K = 0; K < 4; ++K)
		{
			Box += R.A + R.Along * ((K & 1) ? R.L : 0.0) + R.Out * ((K >> 1) ? PitDeep : 0.0);
		}
		Box = Box.ExpandBy(1.0);
		PitBoxes.Add(FBox2D(FVector2D(2.0 * FMath::FloorToDouble(Box.Min.X / 2.0), 2.0 * FMath::FloorToDouble(Box.Min.Y / 2.0)),
			FVector2D(2.0 * FMath::CeilToDouble(Box.Max.X / 2.0), 2.0 * FMath::CeilToDouble(Box.Max.Y / 2.0))));
	}
	auto InPitBox = [&PitBoxes](const FVector2D& P)
	{
		for (const FBox2D& B : PitBoxes)
		{
			if (B.IsInside(P))
			{
				return true;
			}
		}
		return false;
	};
	AddGround(0, NearX0, NearY0, 0.25, FMath::RoundToInt32(NearSize / 0.25), FMath::RoundToInt32(NearSize / 0.25), InPit, Near, NearX0, NearY0, NearSize);
	AddGround(1, G.X0, G.Y0, 2.0, FMath::RoundToInt32(G.Size / 2.0), FMath::RoundToInt32(G.Size / 2.0),
		[&](const FVector2D& P) { return NearBox.IsInside(P) || InPitBox(P); }, Far, G.X0, G.Y0, G.Size);
	for (int32 I = 0; I < PitBoxes.Num(); ++I)
	{
		const FBox2D& B = PitBoxes[I];
		AddGround(2 + I, B.Min.X, B.Min.Y, 0.25, FMath::RoundToInt32((B.Max.X - B.Min.X) / 0.25), FMath::RoundToInt32((B.Max.Y - B.Min.Y) / 0.25),
			[&](const FVector2D& P) { return NearBox.IsInside(P) || InPit(P); }, Far, G.X0, G.Y0, G.Size);
	}
	// The land the park stands on, a step below the field, out to the horizon.
	Shape(Plane, ToUE(G.X0 + G.Size * 0.5, G.Y0 + G.Size * 0.5, -0.3), FVector(60.0 * G.Size, 60.0 * G.Size, 1), BbLook::Land)->GetStaticMeshComponent()->SetCastShadow(false);
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
		Inst(Cube, ToUE(P.X, P.Y, 0.038), FVector(BaseSide, BaseSide, 0.076), BbLook::Rubber, FRotator(0, 45, 0));
	}
	// The rubber: 24 by 6 in, its front edge where the sim has it, on the mound's table.
	const double RubY = D.rubber_m + 0.25 * FT;
	Inst(Cube, ToUE(0, RubY, Ground->Height(0, RubY) + 0.005), FVector(0.5 * FT, 2.0 * FT, 0.03), BbLook::Rubber);
	// Foul poles, each with its screen: a ladder of rungs on the fair side, from above the
	// fence to the top, running along the fence.
	int32 FenceFirst = -1, FenceLast = -1;
	for (uint32 I = 0; I < D.n_wall; ++I)
	{
		if (D.wall[I].kind == 0)
		{
			FenceFirst = FenceFirst < 0 ? (int32)I : FenceFirst;
			FenceLast = (int32)I;
		}
	}
	for (int32 I = 0; I < 2; ++I)
	{
		const FVector2D P(D.poles[I][0], D.poles[I][1]);
		Inst(Cylinder, ToUE(P.X, P.Y, PoleHigh * 0.5), FVector(0.3, 0.3, PoleHigh), BbLook::FoulPole);
		if (FenceFirst < 0)
		{
			continue;
		}
		// Toward fair ground: along the fence's piece that starts (left) or ends (right) here.
		const int32 W = I == 0 ? FenceFirst : FenceLast;
		const FVector2D From(D.wall[W].from[0], D.wall[W].from[1]), To(D.wall[(W + 1) % D.n_wall].from[0], D.wall[(W + 1) % D.n_wall].from[1]);
		const FVector2D Fair = (I == 0 ? To - From : From - To).GetSafeNormal();
		const double Z0 = D.fence_height_m + 0.6, Z1 = PoleHigh - 0.1;
		const FVector2D Edge = P + Fair * PoleScreen;
		Inst(Cylinder, ToUE(Edge.X, Edge.Y, (Z0 + Z1) * 0.5), FVector(0.07, 0.07, Z1 - Z0), BbLook::FoulPole);
		for (double Z = Z0; Z <= Z1 + 0.01; Z += (Z1 - Z0) / 12.0)
		{
			Panel(P, Edge, 0.05, Z - 0.025, Z + 0.025, BbLook::FoulPole);
		}
	}
}

void ABbPark::BuildWalls()
{
	// Every wall stands where the sim has it (physics-spec §9.5), as high as the sim has it,
	// its face toward the field on the sim's boundary and its thickness behind.
	const BbPark& D = Ground->Desc;
	const int32 N = D.n_wall;
	const double Half = WallThick * 0.5;
	for (int32 I = 0; I < N; ++I)
	{
		const BbWallPiece& P = D.wall[I];
		const FVector2D A(P.from[0], P.from[1]), B(D.wall[(I + 1) % N].from[0], D.wall[(I + 1) % N].from[1]);
		// Out of the field: to the left of the piece's direction.
		const FVector2D Along = (B - A).GetSafeNormal(), Out(-Along.Y, Along.X);
		const FVector2D Am = A + Out * Half, Bm = B + Out * Half;
		if (P.kind == 4)
		{
			continue; // a dugout's opening: BuildDugouts
		}
		if (P.kind == 0)
		{
			// The outfield wall: padding, and along its top the line a home run clears.
			const double CapH = 0.12, CapW = WallThick + 0.04;
			Panel(Am, Bm, WallThick, 0.0, P.top_m - CapH, BbLook::WallPad);
			Panel(A + Out * (CapW * 0.5), B + Out * (CapW * 0.5), CapW, P.top_m - CapH, P.top_m, BbLook::WallCap);
		}
		else if (P.top_m > D.foul_wall_height_m + 0.01f)
		{
			// Behind home the walls carry the screen: a low padded wall, and over it the net
			// on its posts, with a rail along the top and a cable half-way up.
			Panel(Am, Bm, WallThick, 0.0, D.foul_wall_height_m, BbLook::WallPad);
			const int32 Bays = FMath::Max(1, FMath::RoundToInt32((B - A).Size() / 7.0));
			for (int32 K = 0; K <= Bays; ++K)
			{
				const FVector2D Q = FMath::Lerp(Am, Bm, (double)K / Bays);
				Inst(Cylinder, ToUE(Q.X, Q.Y, P.top_m * 0.5), FVector(0.12, 0.12, P.top_m), BbLook::Steel);
			}
			Panel(Am, Bm, 0.08, P.top_m - 0.04, P.top_m + 0.04, BbLook::Steel);
			const double MidZ = (D.foul_wall_height_m + P.top_m) * 0.5;
			Panel(Am, Bm, 0.025, MidZ - 0.0125, MidZ + 0.0125, BbLook::Steel, false);
			if (NetMaterial)
			{
				// The one shape here with a material of its own, so an actor of its own.
				const FVector2D Mid = (Am + Bm) * 0.5;
				UStaticMeshComponent* Net = Shape(Cube, ToUE(Mid.X, Mid.Y, (D.foul_wall_height_m + P.top_m) * 0.5), FVector((Bm - Am).Size() + 0.05, 0.02, P.top_m - D.foul_wall_height_m),
					BbLook::Steel, ToUE(Along.X, Along.Y, 0).GetSafeNormal().Rotation())->GetStaticMeshComponent();
				UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(NetMaterial, this);
				M->SetVectorParameterValue(TEXT("Color"), BbLook::Net);
				M->SetScalarParameterValue(TEXT("Opacity"), BbLook::NetOpacity);
				Net->SetMaterial(0, M);
				Net->SetCastShadow(false);
			}
		}
		else
		{
			Panel(Am, Bm, WallThick, 0.0, P.top_m, BbLook::WallPad);
		}
	}
}

void ABbPark::BuildDugouts()
{
	// The sim has the opening and the roof's height, 4 ft above the field: a ball under the
	// roof goes into the dugout. So the dugout is sunk: steps down from the field along the
	// whole opening, a floor, a bench against the rear wall. The ground mesh is open over the
	// pit (BuildSurface) and FBbGround::Height has the steps, so a player walks down them.
	const BbPark& D = Ground->Desc;
	const double Roof = D.dugout_roof_m;
	for (const FBbRun& R : Ground->Dugouts)
	{
		// A box from A0 to A1 along the opening and O0 to O1 back from it.
		auto Box = [&](double A0, double A1, double O0, double O1, double Z0, double Z1, const FBbPaint& Paint, bool bShadow = true)
		{
			const FVector2D Mid = R.Out * ((O0 + O1) * 0.5);
			// (Panel adds 5 cm to a piece's length to close the walls' joints; not wanted here.)
			Panel(R.A + R.Along * (A0 + 0.025) + Mid, R.A + R.Along * (A1 - 0.025) + Mid, O1 - O0, Z0, Z1, Paint, bShadow);
		};
		Box(-PitWall, R.L + PitWall, 0.0, PitDeep + PitWall, PitFloor - 0.2, PitFloor, BbLook::DugoutFloor);
		for (int32 K = 0; K + 1 < PitSteps; ++K)
		{
			Box(0.0, R.L, K * StepRun, (K + 1) * StepRun, PitFloor, -StepRise * (K + 1), BbLook::Concrete);
		}
		// The lip along the field's edge: the top step's riser, and a concrete strip on the field.
		Box(-PitWall, R.L + PitWall, -PitWall, 0.0, PitFloor, 0.012, BbLook::Concrete, false);
		// The end walls, behind the stands' wall and under it; the rear wall; the roof, its top
		// where the sim has it.
		for (double A0 : { -PitWall, R.L })
		{
			Box(A0, A0 + PitWall, WallThick + 0.01, PitDeep + PitWall, PitFloor, Roof, BbLook::DugoutWall);
			Box(A0, A0 + PitWall, 0.0, WallThick + 0.01, PitFloor, -0.002, BbLook::DugoutWall);
		}
		Box(0.0, R.L, PitDeep, PitDeep + PitWall, PitFloor, Roof - RoofThick, BbLook::DugoutWall);
		Box(0.0, R.L, 0.0, PitDeep + PitWall, Roof - RoofThick, Roof, BbLook::Roof);
		// Posts under the roof's front edge, standing on the top step.
		const int32 Bays = FMath::Max(1, FMath::RoundToInt32(R.L / 4.5));
		for (int32 K = 1; K < Bays; ++K)
		{
			const FVector2D Q = R.A + R.Along * (R.L * K / Bays) + R.Out * 0.06;
			Inst(Cylinder, ToUE(Q.X, Q.Y, (Roof - RoofThick - StepRise) * 0.5), FVector(0.08, 0.08, Roof - RoofThick + StepRise), BbLook::Steel);
		}
		// The bench.
		Box(0.6, R.L - 0.6, PitDeep - 0.5, PitDeep - 0.05, PitFloor, PitFloor + 0.45, BbLook::Bench);
	}
}

void ABbPark::BuildOutfield()
{
	// What stands behind the fence, and the figures on it. None of it is in the sim.
	const FBbGround& G = *Ground;
	const BbPark& D = G.Desc;
	const int32 N = D.n_wall;
	// The pens: a wall round each, a bench at the back, the rubbers and the plates.
	for (int32 I = 0; I < N; ++I)
	{
		const BbWallPiece& P = D.wall[I];
		const FVector2D A(P.from[0], P.from[1]), B(D.wall[(I + 1) % N].from[0], D.wall[(I + 1) % N].from[1]);
		const FVector2D Along = (B - A).GetSafeNormal(), Out(-Along.Y, Along.X);
		if (G.Behind[I] == FBbGround::Eye)
		{
			Panel(A + Out * EyeBack - Along * 0.1, B + Out * EyeBack + Along * 0.1, 0.4, 0.0, EyeHigh, BbLook::BatterEye);
		}
		else if (G.Behind[I] == FBbGround::Pen)
		{
			const double Back = PenDeep + WallThick * 0.5;
			Panel(A + Out * Back - Along * 0.2, B + Out * Back + Along * 0.2, WallThick, 0.0, D.fence_height_m, BbLook::WallPad);
			Panel(A + Out * (PenDeep - 0.5) + Along * 0.2, B + Out * (PenDeep - 0.5) - Along * 0.2, 0.4, 0.0, 0.45, BbLook::Bench);
			// Its ends: where the pieces either side have no pen.
			if (G.Behind[(I + N - 1) % N] != FBbGround::Pen)
			{
				Panel(A + Out * WallThick, A + Out * PenDeep, WallThick, 0.0, D.fence_height_m, BbLook::WallPad);
			}
			if (G.Behind[(I + 1) % N] != FBbGround::Pen)
			{
				Panel(B + Out * WallThick, B + Out * PenDeep, WallThick, 0.0, D.fence_height_m, BbLook::WallPad);
			}
		}
	}
	for (int32 K = 0; K < G.PenMounds.Num(); ++K)
	{
		// The pitch runs from the mound to its plate; the rubber and the plate lie across it.
		const FVector2D M = G.PenMounds[K], H = G.PenPlates[K];
		const FVector2D Dir = (H - M).GetSafeNormal(), Across(-Dir.Y, Dir.X);
		Panel(M - Across * 0.28, M + Across * 0.28, 0.5 * FT, 0.0, 0.03, BbLook::Rubber, false);
		Panel(H - Across * 0.19, H + Across * 0.19, 2.0 * PlateHalf, 0.0, 0.012, BbLook::Rubber, false);
	}
	// Distance figures, in feet: at each line (the pole's distance), in each alley and at
	// centre (the fence's distance where the figure stands), each in the middle of a piece.
	if (!TexturedMaterial)
	{
		return;
	}
	const double Bearings[5] = { -90.0, -22.5, 0.0, 22.5, 90.0 };
	for (int32 K = 0; K < 5; ++K)
	{
		// The fence's piece nearest the bearing (the lines: the first and the last).
		int32 Best = -1;
		double BestOff = 1e9;
		FVector2D BestMid = FVector2D::ZeroVector;
		for (int32 I = 0; I < N; ++I)
		{
			if (D.wall[I].kind != 0)
			{
				continue;
			}
			const FVector2D Mid = (FVector2D(D.wall[I].from[0], D.wall[I].from[1]) + FVector2D(D.wall[(I + 1) % N].from[0], D.wall[(I + 1) % N].from[1])) * 0.5;
			const double Off = FMath::Abs(FMath::RadiansToDegrees(FMath::Atan2(Mid.X, Mid.Y)) - Bearings[K]);
			if (Off < BestOff)
			{
				BestOff = Off;
				Best = I;
				BestMid = Mid;
			}
		}
		if (Best < 0)
		{
			continue;
		}
		const FVector2D A(D.wall[Best].from[0], D.wall[Best].from[1]), B(D.wall[(Best + 1) % N].from[0], D.wall[(Best + 1) % N].from[1]);
		const FVector2D Along = (B - A).GetSafeNormal(), Out(-Along.Y, Along.X);
		const double Metres = K == 0 ? G.PoleS[0] : K == 4 ? G.PoleS[1] : BestMid.Size();
		const double W = FMath::Min(MarkW, (B - A).Size() - 0.1), H = MarkH * W / MarkW;
		UTexture2D* Tex = PaintNumber(FMath::RoundToInt32(Metres / FT), MarkW, MarkH, BbLook::MarkerInk, BbLook::WallPad.Color);
		// Seen from the field the fence runs left to right the way the loop does.
		Sign(FVector(BestMid.X, BestMid.Y, FMath::Min(MarkZ, D.wall[Best].top_m * 0.5)) - FVector(Out.X, Out.Y, 0.0) * 0.01, Along, -Out, W, H, Tex, BbLook::WallPad.Rough);
	}
}

UTexture2D* ABbPark::PaintNumber(int32 Value, double W, double H, const FLinearColor& Ink, const FLinearColor& Back)
{
	// Block figures of seven strokes each (top, upper right, lower right, bottom, lower left,
	// upper left, middle), the strokes square-ended so the corners close.
	static const uint8 Strokes[10] = { 63, 6, 91, 79, 102, 109, 125, 7, 127, 111 };
	const FString Text = FString::FromInt(FMath::Max(0, Value));
	const int32 Count = Text.Len();
	const double Dh = H * 0.78, Dw = Dh * 0.55, Gap = Dw * 0.4, T = Dh * 0.17;
	const double X0 = (W - Count * Dw - (Count - 1) * Gap) * 0.5, Y0 = (H - Dh) * 0.5;
	auto Inked = [&](double X, double Y)
	{
		const double U = (X - X0) / (Dw + Gap);
		const int32 K = FMath::FloorToInt32(U);
		if (K < 0 || K >= Count || Y < Y0 || Y > Y0 + Dh)
		{
			return false;
		}
		const double Lx = X - X0 - K * (Dw + Gap), Ly = Y - Y0;
		if (Lx > Dw)
		{
			return false;
		}
		const uint8 S = Strokes[Text[K] - TEXT('0')];
		const bool bLeft = Lx <= T, bRight = Lx >= Dw - T, bUpper = Ly >= Dh * 0.5, bLower = Ly <= Dh * 0.5;
		return ((S & 1) && Ly >= Dh - T) || ((S & 2) && bRight && bUpper) || ((S & 4) && bRight && bLower) || ((S & 8) && Ly <= T)
			|| ((S & 16) && bLeft && bLower) || ((S & 32) && bLeft && bUpper) || ((S & 64) && FMath::Abs(Ly - Dh * 0.5) <= T * 0.5);
	};
	const int32 N = 256;
	TArray<FColor> Top;
	Top.SetNumUninitialized(N * N);
	for (int32 J = 0; J < N; ++J)
	{
		for (int32 I = 0; I < N; ++I)
		{
			int32 Hit = 0;
			for (int32 K = 0; K < 16; ++K)
			{
				// The picture's first row is its top.
				Hit += Inked((I + 0.125 + 0.25 * (K & 3)) * W / N, H - (J + 0.125 + 0.25 * (K >> 2)) * H / N) ? 1 : 0;
			}
			FLinearColor C = FMath::Lerp(Back, Ink, Hit / 16.f);
			C.A = 1.f;
			Top[J * N + I] = C.ToFColor(true);
		}
	}
	return MakeTexture(MoveTemp(Top), N);
}

void ABbPark::Sign(const FVector& Mid, const FVector2D& Right, const FVector2D& Face, double W, double H, UTexture2D* Tex, float Rough)
{
	if (!Signs)
	{
		Signs = NewObject<UProceduralMeshComponent>(this);
		Signs->SetupAttachment(RootComponent);
		Signs->SetMobility(EComponentMobility::Movable);
		Signs->SetCollisionEnabled(ECollisionEnabled::NoCollision);
		Signs->SetCastShadow(false);
		Signs->RegisterComponent();
	}
	// Top left, top right, bottom left, bottom right, as the picture's rows run.
	TArray<FVector> Verts;
	for (int32 K = 0; K < 4; ++K)
	{
		const double U = (K & 1) ? 0.5 : -0.5, V = (K >> 1) ? -0.5 : 0.5;
		Verts.Add(ToUE(Mid.X + Right.X * U * W, Mid.Y + Right.Y * U * W, Mid.Z + V * H));
	}
	const FVector Normal = ToUE(Face.X, Face.Y, 0.0).GetSafeNormal();
	TArray<int32> Tris = { 0, 1, 2, 2, 1, 3 };
	// The side that shows is the one the corners run round the way the ground's do (AddGround).
	if ((((Verts[1] - Verts[0]) ^ (Verts[2] - Verts[0])) | Normal) > 0.0)
	{
		Tris = { 0, 2, 1, 2, 3, 1 };
	}
	const TArray<FVector> Normals = { Normal, Normal, Normal, Normal };
	const TArray<FVector2D> UVs = { FVector2D(0, 0), FVector2D(1, 0), FVector2D(0, 1), FVector2D(1, 1) };
	const FVector Tan = ToUE(Right.X, Right.Y, 0.0).GetSafeNormal();
	TArray<FProcMeshTangent> Tangents;
	Tangents.Init(FProcMeshTangent(Tan, false), 4);
	const int32 Section = SignCount++;
	Signs->CreateMeshSection_LinearColor(Section, Verts, Tris, Normals, UVs, TArray<FLinearColor>(), Tangents, false);
	UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(TexturedMaterial, this);
	M->SetTextureParameterValue(TEXT("Tex"), Tex);
	M->SetScalarParameterValue(TEXT("Rough"), Rough);
	Signs->SetMaterial(Section, M);
}

/** Flat faces gathered for one paint, drawn as one piece of the bowl's mesh. */
struct FBbSolid
{
	TArray<FVector> V, N;
	TArray<int32> T;
	/** A flat four-cornered face (sim frame, metres): P0 to P1 is one edge and P2 to P3 the one
	 *  opposite; it shows from the side Normal points to. */
	void Quad(const FVector& P0, const FVector& P1, const FVector& P2, const FVector& P3, const FVector& Normal)
	{
		const int32 B = V.Num();
		for (const FVector& P : { P0, P1, P2, P3 })
		{
			V.Add(ToUE(P.X, P.Y, P.Z));
		}
		const FVector Nu = ToUE(Normal.X, Normal.Y, Normal.Z).GetSafeNormal();
		N.Append({ Nu, Nu, Nu, Nu });
		// The side that shows is the one the corners run round the way the ground's do (AddGround).
		if ((((V[B + 1] - V[B]) ^ (V[B + 2] - V[B])) | Nu) > 0.0)
		{
			T.Append({ B, B + 2, B + 1, B + 2, B + 3, B + 1 });
		}
		else
		{
			T.Append({ B, B + 1, B + 2, B + 2, B + 1, B + 3 });
		}
	}
};

namespace
{
	// The seating bowl (park-visuals step 4). None of it is in the sim: the sim has the wall
	// the seats stand behind, and everything here follows that wall by rule.
	const double LandZ = -0.3;                  // the land outside the park
	const int32 BowlRows = 18;
	const double RowRun = 0.84, RowRise = 0.36; // a row's depth and its step up
	const double Parapet = 0.85;                // the wall's top above the first row's floor
	const double Concourse = 3.6, RailHigh = 1.1, RailThick = 0.25;
	const double SectionLong = 9.0, AisleHalf = 0.6;
	const double SeatPitch = 0.52, SeatWide = 0.44, SeatDeep = 0.42, SeatHigh = 0.42, SeatBack = 0.45;
	// The roof over the concourse and the last rows: how far forward of the top row it
	// reaches, its height above the concourse at its front and rear edges, how far it runs
	// past the back wall.
	const double RoofOver = 5.5, RoofFront = 5.0, RoofRear = 4.2, RoofSlab = 0.25, RoofPast = 0.8;
	const double PostWide = 0.16;
	// The pressbox, on the roof behind home.
	const double PressHigh = 3.6, PressWideMax = 26.0, PressFront = 2.0, PressBehind = 3.0;
	// Beyond the fence: a walk at its foot, then a grass bank in left field and benches in right.
	const double OutWalk = 2.5;
	const double BermFoot = 0.9, BermRun = 12.0, BermTop = 5.2, BermPath = 2.5;
	const int32 BenchRows = 10;
	const double BenchRun = 0.8, BenchRise = 0.42, BenchFoot = 1.6, BenchWalk = 2.0;
	const int32 BenchAisleEvery = 5;            // an aisle at every fifth joint of the fence
	// Light towers, by their bearing from home (degrees from centre field, either side).
	const double TowerDeg[3] = { 32.0, 78.0, 125.0 };
	const double TowerHigh = 32.0, TowerBehind = 2.5, TowerWide = 0.9;
	const double RackWide = 7.0, RackHigh = 4.5;
	const int32 LampRows = 3, LampCols = 6;

	/** A straight piece of something that follows the boundary. A point D metres behind the
	 *  wall is A + MA * D at one end and B + MB * D at the other: where two pieces meet at a
	 *  corner both ends lie on the line that halves it, so they close without a gap. */
	struct FBbPiece
	{
		FVector2D A, B, Along, Out, MA, MB;
		double L;
		bool bCapA, bCapB;
		FVector2D Row(double D, double T) const { return FMath::Lerp(A + MA * D, B + MB * D, T); }
	};

	/** The faces of a section (points of distance behind the wall and height) carried along a
	 *  piece. Each face shows from the left of the way the section is drawn. */
	void Extrude(FBbSolid& S, const FBbPiece& P, TConstArrayView<FVector2D> Prof)
	{
		for (int32 I = 0; I + 1 < Prof.Num(); ++I)
		{
			const FVector2D& U = Prof[I];
			const FVector2D& W = Prof[I + 1];
			const FVector2D Nm = FVector2D(U.Y - W.Y, W.X - U.X).GetSafeNormal();
			if (Nm.IsZero())
			{
				continue;
			}
			const FVector2D A0 = P.A + P.MA * U.X, B0 = P.B + P.MB * U.X, A1 = P.A + P.MA * W.X, B1 = P.B + P.MB * W.X;
			S.Quad(FVector(A0.X, A0.Y, U.Y), FVector(B0.X, B0.Y, U.Y), FVector(A1.X, A1.Y, W.Y), FVector(B1.X, B1.Y, W.Y), FVector(P.Out.X * Nm.X, P.Out.Y * Nm.X, Nm.Y));
		}
	}

	/** The end of a section that stands on the ground: closed from the section down to the land. */
	void CapDown(FBbSolid& S, const FVector2D& Base, const FVector2D& M, TConstArrayView<FVector2D> Prof, const FVector2D& Face)
	{
		for (int32 I = 0; I + 1 < Prof.Num(); ++I)
		{
			const FVector2D& U = Prof[I];
			const FVector2D& W = Prof[I + 1];
			if (FMath::Abs(W.X - U.X) < 1e-6)
			{
				continue;
			}
			const FVector2D P0 = Base + M * U.X, P1 = Base + M * W.X;
			S.Quad(FVector(P0.X, P0.Y, LandZ), FVector(P1.X, P1.Y, LandZ), FVector(P0.X, P0.Y, U.Y), FVector(P1.X, P1.Y, W.Y), FVector(Face.X, Face.Y, 0.0));
		}
	}

	double Cross2(const FVector2D& A, const FVector2D& B)
	{
		return A.X * B.Y - A.Y * B.X;
	}
}

void ABbPark::Inst(UStaticMesh* Mesh, const FVector& Location, const FVector& Scale, const FBbPaint& Paint, const FRotator& Rotation, bool bShadow)
{
	FBbBatch* Found = Batches.FindByPredicate([&](const FBbBatch& B)
	{
		return B.Mesh == Mesh && B.bShadow == bShadow && B.Paint.Color == Paint.Color && B.Paint.Rough == Paint.Rough;
	});
	if (!Found)
	{
		Found = &Batches.Add_GetRef(FBbBatch{ Mesh, Paint, bShadow, {} });
	}
	Found->At.Add(FTransform(Rotation, Location, Scale));
}

void ABbPark::InstPanel(const FVector2D& A, const FVector2D& B, double Depth, double Z0, double Z1, const FBbPaint& Paint, bool bShadow)
{
	const FVector2D Mid = (A + B) * 0.5;
	const FVector Dir = ToUE(B.X - A.X, B.Y - A.Y, 0).GetSafeNormal();
	Inst(Cube, ToUE(Mid.X, Mid.Y, (Z0 + Z1) * 0.5), FVector((B - A).Size(), Depth, Z1 - Z0), Paint, Dir.Rotation(), bShadow);
}

UMaterialInstanceDynamic* ABbPark::PaintOf(const FBbPaint& Paint)
{
	UMaterialInterface* Base = PaintMaterial ? PaintMaterial.Get() : FlatMaterial.Get();
	if (!Base)
	{
		return nullptr;
	}
	UMaterialInstanceDynamic* M = UMaterialInstanceDynamic::Create(Base, this);
	M->SetVectorParameterValue(TEXT("Color"), Paint.Color);
	M->SetScalarParameterValue(TEXT("Rough"), Paint.Rough);
	return M;
}

void ABbPark::FlushInst()
{
	int32 Count = 0;
	for (const FBbBatch& B : Batches)
	{
		UInstancedStaticMeshComponent* C = NewObject<UInstancedStaticMeshComponent>(this);
		C->SetupAttachment(RootComponent);
		C->SetMobility(EComponentMobility::Movable);
		C->SetStaticMesh(B.Mesh);
		C->SetCollisionEnabled(ECollisionEnabled::NoCollision);
		C->SetCastShadow(B.bShadow);
		if (UMaterialInstanceDynamic* M = PaintOf(B.Paint))
		{
			C->SetMaterial(0, M);
		}
		C->RegisterComponent();
		C->AddInstances(B.At, false, true);
		Count += B.At.Num();
	}
	UE_LOG(LogBbPark, Display, TEXT("BB_PARK stands: %d shapes in %d batches, %d faces"), Count, Batches.Num(), BowlFaces);
	Batches.Reset();
}

void ABbPark::AddSolid(const FBbSolid& S, const FBbPaint& Paint)
{
	if (S.V.Num() == 0)
	{
		return;
	}
	if (!Bowl)
	{
		Bowl = NewObject<UProceduralMeshComponent>(this);
		Bowl->SetupAttachment(RootComponent);
		Bowl->SetMobility(EComponentMobility::Movable);
		Bowl->SetCollisionEnabled(ECollisionEnabled::NoCollision);
		Bowl->RegisterComponent();
	}
	const int32 Section = BowlSections++;
	Bowl->CreateMeshSection_LinearColor(Section, S.V, S.T, S.N, TArray<FVector2D>(), TArray<FLinearColor>(), TArray<FProcMeshTangent>(), false);
	if (UMaterialInstanceDynamic* M = PaintOf(Paint))
	{
		Bowl->SetMaterial(Section, M);
	}
	BowlFaces += S.V.Num() / 4;
}

void ABbPark::BuildStands()
{
	// One deck of seats behind the walls of foul ground, a roof over its concourse, the
	// pressbox behind home, a grass bank beyond the left-field fence and benches beyond the
	// right, and six light towers (unlit). None of it is in the sim.
	const FBbGround& G = *Ground;
	const BbPark& D = G.Desc;
	const int32 N = D.n_wall;
	TArray<FVector2D> At, Along, Out;
	TArray<double> Len;
	for (int32 I = 0; I < N; ++I)
	{
		const FVector2D A(D.wall[I].from[0], D.wall[I].from[1]), B(D.wall[(I + 1) % N].from[0], D.wall[(I + 1) % N].from[1]);
		At.Add(A);
		Along.Add((B - A).GetSafeNormal());
		Out.Add(FVector2D(-Along[I].Y, Along[I].X));
		Len.Add((B - A).Size());
	}
	// A piece that is joined to its neighbour at an end shares the corner with it; one that
	// is not ends square.
	auto Make = [&](int32 I, bool bJoinA, bool bJoinB)
	{
		const int32 P = (I + N - 1) % N, Q = (I + 1) % N;
		FBbPiece R;
		R.A = At[I];
		R.B = At[Q];
		R.Along = Along[I];
		R.Out = Out[I];
		R.L = Len[I];
		R.MA = bJoinA ? (Out[P] + Out[I]) / (1.0 + (Out[P] | Out[I])) : Out[I];
		R.MB = bJoinB ? (Out[Q] + Out[I]) / (1.0 + (Out[Q] | Out[I])) : Out[I];
		R.bCapA = !bJoinA;
		R.bCapB = !bJoinB;
		return R;
	};
	FBbSolid Concrete, Grass, RoofFaces;

	// ---- The deck -----------------------------------------------------------------------
	// Rows step up and back from the wall, whose top is a parapet in front of the first row.
	// Over a dugout the first rows are left out: its roof is there, and the seats start
	// behind it (the sim: a ball over the roof goes into the seats).
	const double Z0 = D.foul_wall_height_m - Parapet;
	const double DTop = WallThick + BowlRows * RowRun, ZTop = Z0 + BowlRows * RowRise, DBack = DTop + Concourse;
	const int32 DugRow = FMath::Min(BowlRows - 1, FMath::CeilToInt32((PitDeep + PitWall - WallThick) / RowRun));
	auto Foul = [&D, N](int32 I) { return D.wall[(I + N) % N].kind != 0; };
	auto Dug = [&D, N](int32 I) { return D.wall[(I + N) % N].kind == 4; };
	auto Roofed = [&D, N](int32 I) { const uint8 K = D.wall[(I + N) % N].kind; return K != 0 && K != 3; };
	const double RoofD0 = DTop - RoofOver, RoofD1 = DBack + RoofPast;
	auto RoofUnder = [&](double Dd) { return ZTop + FMath::Lerp(RoofFront, RoofRear, (Dd - RoofD0) / (RoofD1 - RoofD0)); };
	int32 Seats = 0;
	for (int32 I = 0; I < N; ++I)
	{
		if (!Foul(I))
		{
			continue;
		}
		const bool bDug = Dug(I);
		const int32 K0 = bDug ? DugRow : 0;
		FBbPiece R = Make(I, Foul(I - 1), Foul(I + 1));
		R.bCapA |= Dug(I - 1) != bDug;
		R.bCapB |= Dug(I + 1) != bDug;
		TArray<FVector2D> Prof;
		if (bDug)
		{
			Prof.Add(FVector2D(PitDeep + PitWall - 0.02, D.dugout_roof_m));
			Prof.Add(FVector2D(WallThick + K0 * RowRun, D.dugout_roof_m));
		}
		for (int32 K = K0; K < BowlRows; ++K)
		{
			Prof.Add(FVector2D(WallThick + K * RowRun, Z0 + K * RowRise));
			Prof.Add(FVector2D(WallThick + (K + 1) * RowRun, Z0 + K * RowRise));
		}
		// The concourse behind the top row, its rail, and the back wall down to the land.
		Prof.Append({ FVector2D(DTop, ZTop), FVector2D(DBack, ZTop), FVector2D(DBack, ZTop + RailHigh), FVector2D(DBack + RailThick, ZTop + RailHigh), FVector2D(DBack + RailThick, LandZ) });
		Extrude(Concrete, R, Prof);
		if (R.bCapA)
		{
			CapDown(Concrete, R.A, R.MA, Prof, -R.Along);
		}
		if (R.bCapB)
		{
			CapDown(Concrete, R.B, R.MB, Prof, R.Along);
		}
		// Sections of seats between aisles; an aisle at every corner too. Each seat is a
		// bottom and a back; each aisle has a half step on every row.
		const int32 Sections = FMath::Max(1, FMath::RoundToInt32(R.L / SectionLong));
		const FRotator Rot = ToUE(R.Along.X, R.Along.Y, 0).GetSafeNormal().Rotation();
		for (int32 K = K0; K < BowlRows; ++K)
		{
			const double Dk = WallThick + K * RowRun, Zk = Z0 + K * RowRise;
			const double Ds = Dk + 0.45, Lr = (R.Row(Ds, 1.0) - R.Row(Ds, 0.0)).Size();
			for (int32 J = 0; J < Sections; ++J)
			{
				const double S0 = Lr * J / Sections + AisleHalf, S1 = Lr * (J + 1) / Sections - AisleHalf;
				const int32 Count = FMath::FloorToInt32((S1 - S0) / SeatPitch);
				for (int32 C = 0; C < Count; ++C)
				{
					const FVector2D P = R.Row(Ds, ((S0 + S1) * 0.5 + (C - (Count - 1) * 0.5) * SeatPitch) / Lr);
					const FVector2D Pb = P + R.Out * (SeatDeep * 0.5);
					Inst(Cube, ToUE(P.X, P.Y, Zk + SeatHigh), FVector(SeatWide, SeatDeep, 0.06), BbLook::Seats, Rot, false);
					Inst(Cube, ToUE(Pb.X, Pb.Y, Zk + SeatHigh + SeatBack * 0.5), FVector(SeatWide, 0.06, SeatBack), BbLook::Seats, Rot, false);
					++Seats;
				}
			}
			const double Dh = Dk + RowRun - 0.21, Lh = (R.Row(Dh, 1.0) - R.Row(Dh, 0.0)).Size();
			for (int32 J = 0; J <= Sections; ++J)
			{
				const double S = FMath::Clamp(Lh * J / Sections, AisleHalf * 0.5, Lh - AisleHalf * 0.5);
				const double Wide = (J == 0 || J == Sections) ? AisleHalf : 2.0 * AisleHalf;
				const FVector2D P = R.Row(Dh, S / Lh);
				Inst(Cube, ToUE(P.X, P.Y, Zk + RowRise * 0.25), FVector(Wide, 0.42, RowRise * 0.5), BbLook::Concrete, Rot, false);
			}
		}
		// The roof: a slab higher at the front, on posts at the aisles. It stops where the
		// stands close in to the poles.
		if (!Roofed(I))
		{
			continue;
		}
		const FBbPiece T = Make(I, Roofed(I - 1), Roofed(I + 1));
		const FVector2D Slab[5] = { FVector2D(RoofD0, RoofUnder(RoofD0) + RoofSlab), FVector2D(RoofD1, RoofUnder(RoofD1) + RoofSlab), FVector2D(RoofD1, RoofUnder(RoofD1)), FVector2D(RoofD0, RoofUnder(RoofD0)), FVector2D(RoofD0, RoofUnder(RoofD0) + RoofSlab) };
		Extrude(RoofFaces, T, Slab);
		for (int32 E = 0; E < 2; ++E)
		{
			if (!(E == 0 ? T.bCapA : T.bCapB))
			{
				continue;
			}
			FVector C[4];
			for (int32 K = 0; K < 4; ++K)
			{
				const FVector2D P = T.Row(Slab[K].X, E);
				C[K] = FVector(P.X, P.Y, Slab[K].Y);
			}
			RoofFaces.Quad(C[0], C[1], C[3], C[2], FVector(T.Along.X, T.Along.Y, 0.0) * (E == 0 ? -1.0 : 1.0));
		}
		for (int32 J = 0; J < Sections + (T.bCapB ? 1 : 0); ++J)
		{
			for (int32 Rear = 0; Rear < 2; ++Rear)
			{
				const double Dd = Rear ? DBack + RailThick * 0.5 : DTop + 0.25;
				const double Zf = ZTop + (Rear ? RailHigh : 0.0), Zr = RoofUnder(Dd);
				const FVector2D P = T.Row(Dd, (double)J / Sections);
				Inst(Cylinder, ToUE(P.X, P.Y, (Zf + Zr) * 0.5), FVector(PostWide, PostWide, Zr - Zf), BbLook::Steel, FRotator::ZeroRotator, false);
			}
		}
	}

	// ---- The pressbox ---------------------------------------------------------------------
	// On the roof behind home, on the backstop's piece nearest the line through the plate: a
	// room with a band of glass toward the field, and its building behind the concourse.
	int32 Home = -1;
	for (int32 I = 0; I < N; ++I)
	{
		if (D.wall[I].kind == 1 && (Home < 0 || FMath::Abs(At[I].X + At[(I + 1) % N].X) < FMath::Abs(At[Home].X + At[(Home + 1) % N].X)))
		{
			Home = I;
		}
	}
	if (Home >= 0)
	{
		const FBbPiece R = Make(Home, Foul(Home - 1), Foul(Home + 1));
		const double D0 = DTop - PressFront, D1 = RoofD1, D2 = RoofD1 + PressBehind;
		const double Wide = FMath::Min(PressWideMax, (R.Row(D0, 1.0) - R.Row(D0, 0.0)).Size() - 1.0);
		const double Zb = RoofUnder(RoofD1), Zt = RoofUnder(D0) + RoofSlab + PressHigh;
		auto Slab = [&](double Da, double Db, double W, double Za, double Zc, const FBbPaint& Paint, bool bShadow = true)
		{
			const FVector2D C = R.Row((Da + Db) * 0.5, 0.5);
			InstPanel(C - R.Along * (W * 0.5), C + R.Along * (W * 0.5), Db - Da, Za, Zc, Paint, bShadow);
		};
		if (Wide > 4.0)
		{
			Slab(D0, D1, Wide, Zb, Zt, BbLook::Pressbox);
			Slab(DBack + RailThick, D2, Wide, LandZ, Zt, BbLook::Pressbox);
			Slab(D0 - 0.4, D2 + 0.4, Wide + 0.8, Zt, Zt + 0.2, BbLook::Roof);
			Slab(D0 - 0.04, D0, Wide - 1.0, Zt - 2.3, Zt - 0.7, BbLook::Glass, false);
		}
	}

	// ---- Beyond the fence -----------------------------------------------------------------
	// Left field: a grass bank to sit on, a path along its top. Right field: benches. Both
	// leave the batter's eye and the pens clear, and end square beside them and at the poles.
	for (int32 Side = 0; Side < 2; ++Side)
	{
		auto Here = [&](int32 I)
		{
			I = (I + N) % N;
			return D.wall[I].kind == 0 && G.Behind[I] == FBbGround::Plain && ((At[I].X + At[(I + 1) % N].X < 0.0) == (Side == 0));
		};
		for (int32 I = 0; I < N; ++I)
		{
			if (!Here(I))
			{
				continue;
			}
			const FBbPiece R = Make(I, Here(I - 1), Here(I + 1));
			TArray<FVector2D> Prof;
			if (Side == 0)
			{
				const double Dt = OutWalk + BermRun;
				Prof = { FVector2D(OutWalk, LandZ), FVector2D(OutWalk, BermFoot), FVector2D(Dt, BermTop), FVector2D(Dt + BermPath, BermTop), FVector2D(Dt + BermPath, LandZ) };
				Extrude(Concrete, R, MakeArrayView(Prof.GetData(), 2));
				Extrude(Grass, R, MakeArrayView(Prof.GetData() + 1, 2));
				Extrude(Concrete, R, MakeArrayView(Prof.GetData() + 2, 3));
			}
			else
			{
				Prof.Add(FVector2D(OutWalk, LandZ));
				for (int32 K = 0; K < BenchRows; ++K)
				{
					Prof.Add(FVector2D(OutWalk + K * BenchRun, BenchFoot + K * BenchRise));
					Prof.Add(FVector2D(OutWalk + (K + 1) * BenchRun, BenchFoot + K * BenchRise));
				}
				const double Dt = OutWalk + BenchRows * BenchRun, Zt = BenchFoot + BenchRows * BenchRise;
				Prof.Append({ FVector2D(Dt, Zt), FVector2D(Dt + BenchWalk, Zt), FVector2D(Dt + BenchWalk, Zt + RailHigh), FVector2D(Dt + BenchWalk + RailThick, Zt + RailHigh), FVector2D(Dt + BenchWalk + RailThick, LandZ) });
				Extrude(Concrete, R, Prof);
				// A bench along each row, broken for an aisle at every few joints and at the ends.
				const double GapA = (R.bCapA || I % BenchAisleEvery == 0) ? AisleHalf : 0.0, GapB = (R.bCapB || (I + 1) % BenchAisleEvery == 0) ? AisleHalf : 0.0;
				for (int32 K = 0; K < BenchRows; ++K)
				{
					const double Dd = OutWalk + K * BenchRun + 0.42, Zk = BenchFoot + K * BenchRise;
					const double Lr = (R.Row(Dd, 1.0) - R.Row(Dd, 0.0)).Size();
					if (Lr > GapA + GapB + 0.3)
					{
						InstPanel(R.Row(Dd, GapA / Lr), R.Row(Dd, 1.0 - GapB / Lr), 0.28, Zk + 0.42, Zk + 0.47, BbLook::Bench2, false);
					}
				}
			}
			if (R.bCapA)
			{
				CapDown(Concrete, R.A, R.MA, Prof, -R.Along);
			}
			if (R.bCapB)
			{
				CapDown(Concrete, R.B, R.MB, Prof, R.Along);
			}
		}
	}

	// ---- Light towers (unlit: night games are a later stage) -------------------------------
	// Behind whatever stands at each bearing from home: a pole, and at its top a rack of
	// lamps turned to the middle of the field.
	const FVector2D Middle(0.0, D.rubber_m + 15.0);
	for (double Deg : TowerDeg)
	{
		for (double Hand : { -1.0, 1.0 })
		{
			const double Th = FMath::DegreesToRadians(Deg * Hand);
			const FVector2D Ray(FMath::Sin(Th), FMath::Cos(Th));
			for (int32 I = 0; I < N; ++I)
			{
				const FVector2D E = At[(I + 1) % N] - At[I];
				const double Den = Cross2(Ray, E);
				if (FMath::Abs(Den) < 1e-9)
				{
					continue;
				}
				const double U = Cross2(At[I], E) / Den, T = Cross2(At[I], Ray) / Den;
				if (U <= 0.0 || T < 0.0 || T >= 1.0)
				{
					continue;
				}
				const double Behind = D.wall[I].kind == 0 ? OutWalk + BermRun + BermPath : DBack + RailThick;
				const FVector2D P = At[I] + E * T + Out[I] * (Behind + TowerBehind);
				const FVector2D F = (Middle - P).GetSafeNormal(), Across(-F.Y, F.X);
				Inst(Cylinder, ToUE(P.X, P.Y, (TowerHigh + LandZ) * 0.5), FVector(TowerWide, TowerWide, TowerHigh - LandZ), BbLook::Steel);
				const FVector2D C = P + F * (TowerWide * 0.5 + 0.2);
				InstPanel(C - Across * (RackWide * 0.5), C + Across * (RackWide * 0.5), 0.4, TowerHigh - RackHigh, TowerHigh, BbLook::Roof);
				for (int32 Row = 0; Row < LampRows; ++Row)
				{
					for (int32 Col = 0; Col < LampCols; ++Col)
					{
						const FVector2D L = C + F * 0.25 + Across * ((Col - (LampCols - 1) * 0.5) * RackWide / LampCols);
						const double Z = TowerHigh - RackHigh * (Row + 0.5) / LampRows;
						InstPanel(L - Across * 0.4, L + Across * 0.4, 0.12, Z - 0.45, Z + 0.45, BbLook::Lamp, false);
					}
				}
				break;
			}
		}
	}

	AddSolid(Concrete, BbLook::Concrete);
	AddSolid(Grass, BbLook::Berm);
	AddSolid(RoofFaces, BbLook::Roof);
	UE_LOG(LogBbPark, Display, TEXT("BB_PARK seats: %d"), Seats);
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
	// From a seat half-way up the deck behind third (the roof and a light tower stand where
	// this spot was before the seating bowl was built).
	S.Add(At(TEXT("third_stands"), ToUE(-31, -3, 5.4), ToUE(5, 25, 0), 65.f));
	S.Add(At(TEXT("mound"), ToUE(4.5, Desc.rubber_m - 7.0, 0.9), ToUE(0, Desc.rubber_m - 0.3, 0.35), 40.f));
	// Straight down, centre field at the top of the picture, the whole park in a 16:9 frame.
	const double MidY = (Deep - 20.0) * 0.5;
	const double Half = FMath::Max((Deep + 30.0) * 0.5, Wide * 9.0 / 16.0 + 8.0);
	const float PlanFov = 50.f;
	const double High = Half / (FMath::Tan(FMath::DegreesToRadians(PlanFov * 0.5)) * 9.0 / 16.0);
	S.Add(FBbParkSpot{ TEXT("plan"), ToUE(0, MidY, High), FRotator(-90.f, 0.f, 0.f), PlanFov });
	return S;
}
