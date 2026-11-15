// The ballpark: everything that stands still. Built in code from the sim's description of
// the park (bb_game_park, docs/park-visuals.md); nothing here is in the game.
#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "bb_ffi.h"
#include "BbLook.h"
#include "BbPark.generated.h"

class AStaticMeshActor;
class UStaticMesh;
class UMaterialInterface;
class UTexture2D;
class UProceduralMeshComponent;
class UMaterialInstanceDynamic;
struct FBbSolid;

/** A fixed place to look at the park from (park-visuals §4): the same spots before and after
 *  every change to its looks. */
struct FBbParkSpot
{
	const TCHAR* Name;
	FVector Eye;
	FRotator Look;
	float Fov;
};

/**
 * Where things are on the playing surface, in the sim frame (metres). The boundary, bases,
 * rubber and poles are the sim's. The groundskeeping (which ground is dirt, where the chalk
 * goes, the mound's shape) follows the Official Baseball Rules' field diagram, which is the
 * same in every park, laid out on the sim's bases and rubber.
 *
 * Two things the drawing has and the physics does not (park-visuals §2): the mound's height
 * (the sim's ground is flat) and the shape of the infield dirt (the sim's is a disc around
 * home). Neither is read back by anything.
 */
/** A straight piece of the boundary: from A for L metres along Along; Out points out of the field. */
struct FBbRun
{
	FVector2D A, Along, Out;
	double L;
};

struct FBbGround
{
	explicit FBbGround(const BbPark& Desc);

	/** Height of the drawn ground above the sim's flat field, m: the mound. */
	double Height(double X, double Y) const;
	/** The ground's colour at a point (linear). */
	FLinearColor Color(double X, double Y) const;
	/** Signed distance to the park's boundary (positive inside the field), m. */
	float Inside(double X, double Y) const;

	BbPark Desc;
	/** The square that holds the whole park, on a 2 m grid. */
	double X0 = 0.0, Y0 = 0.0, Size = 0.0;
	double BaseL = 0.0;
	double PoleS[2] = { 0.0, 0.0 };
	FVector2D MoundC = FVector2D::ZeroVector, SkinC = FVector2D::ZeroVector, OnDeck[2];
	/** The dugouts' openings. Behind each the ground steps down to the dugout's floor. */
	TArray<FBbRun> Dugouts;
	/** What stands behind each piece of the fence: nothing of the field's, the batter's eye,
	 *  or a bullpen. Decided by rule from the piece's bearing, so any park gets them. */
	enum : uint8 { Plain, Eye, Pen };
	uint8 Behind[BB_PARK_MAX_WALL];
	/** The fence's pieces with a bullpen behind them, and where each pen's two mounds and
	 *  two plates are (empty if the park has no room for a pen). */
	TArray<FBbRun> Pens;
	TArray<FVector2D> PenMounds, PenPlates;

private:
	void Lookup(double X, double Y, float& Sd, float& Fence) const;
	/** Signed distance to the boundary and distance to the fence, on a grid over the square. */
	int32 Cells = 0;
	TArray<float> Sd, Fd;
};

UCLASS()
class ABbPark : public AActor
{
	GENERATED_BODY()

public:
	ABbPark();
	/** Sun and sky. Needs no park, so a scene without a game is lit too. */
	void BuildLights();
	/** The field, its walls and the stands, from the sim's park. */
	void Build(const BbPark& Desc);
	/** Height of the drawn ground above the sim's flat field at a point, m (0 before Build). */
	double GroundZ(double X, double Y) const { return Ground ? Ground->Height(X, Y) : 0.0; }
	/** A step inside the dugout of a side (+1 first base, -1 third base): where a player
	 *  walking off disappears under the roof. */
	FVector2D DugoutDoor(double Side) const;

	/** The fixed camera spots for this park: behind home, centre field looking in, high on the
	 *  first-base side, the stands behind third, low at the mound, and straight down. */
	static TArray<FBbParkSpot> Spots(const BbPark& Desc);

private:
	AStaticMeshActor* Shape(UStaticMesh* Mesh, const FVector& Location, const FVector& Scale, const FBbPaint& Paint, const FRotator& Rotation = FRotator::ZeroRotator);
	/** A box standing on the line from A to B (sim frame), Depth across and from Z0 to Z1,
	 *  a touch longer than the line so that walls close at their joints. Drawn as Inst's are. */
	void Panel(const FVector2D& A, const FVector2D& B, double Depth, double Z0, double Z1, const FBbPaint& Paint, bool bShadow = true);
	void BuildSurface();
	void BuildFurniture();
	void BuildWalls();
	void BuildDugouts();
	void BuildOutfield();
	void BuildStands();
	/** One of many alike (a seat, a post, a lamp): drawn together with every other shape of
	 *  its mesh and paint, once FlushInst has run. Sizes in metres, as Shape's. */
	void Inst(UStaticMesh* Mesh, const FVector& Location, const FVector& Scale, const FBbPaint& Paint, const FRotator& Rotation = FRotator::ZeroRotator, bool bShadow = true);
	/** Panel's box at exactly the line's length. */
	void InstPanel(const FVector2D& A, const FVector2D& B, double Depth, double Z0, double Z1, const FBbPaint& Paint, bool bShadow = true);
	void FlushInst();
	/** Faces gathered for one paint become one piece of the bowl's mesh. */
	void AddSolid(const FBbSolid& S, const FBbPaint& Paint);
	UMaterialInstanceDynamic* PaintOf(const FBbPaint& Paint);
	/** A flat picture standing on a wall: centred at Mid (sim frame), W wide along Right and
	 *  H high, seen from the side Face points to. */
	void Sign(const FVector& Mid, const FVector2D& Right, const FVector2D& Face, double W, double H, UTexture2D* Tex, float Rough);
	/** A whole number in block figures, W by H metres, on a ground colour. */
	UTexture2D* PaintNumber(int32 Value, double W, double H, const FLinearColor& Ink, const FLinearColor& Back);
	/** A square picture with its smaller copies. */
	UTexture2D* MakeTexture(TArray<FColor>&& Top, int32 N);
	/** The ground's colours over a square, painted in code, with its smaller copies. */
	UTexture2D* PaintGround(double TexX0, double TexY0, double TexSize, int32 N);
	/** One piece of the ground mesh: a grid of cells whose heights follow the drawn ground,
	 *  leaving out the cells whose middle Skip says to. */
	void AddGround(int32 Section, double GX0, double GY0, double Cell, int32 Nx, int32 Ny, TFunctionRef<bool(const FVector2D&)> Skip, UTexture2D* Tex, double TexX0, double TexY0, double TexSize);

	TUniquePtr<FBbGround> Ground;
	UPROPERTY()
	TObjectPtr<UStaticMesh> Cube;
	UPROPERTY()
	TObjectPtr<UStaticMesh> Cylinder;
	UPROPERTY()
	TObjectPtr<UStaticMesh> Plane;
	/** Paint: a colour and how matt it is (parameters "Color", "Rough"), built in code. */
	UPROPERTY()
	TObjectPtr<UMaterialInterface> PaintMaterial;
	/** The engine's flat-colour material (parameter "Color"): what is used without the above. */
	UPROPERTY()
	TObjectPtr<UMaterialInterface> FlatMaterial;
	/** See-through and unlit (parameters "Color", "Opacity"): the backstop's net. */
	UPROPERTY()
	TObjectPtr<UMaterialInterface> NetMaterial;
	/** A lit material showing a texture (parameters "Tex", "Rough"), built in code at start-up. */
	UPROPERTY()
	TObjectPtr<UMaterialInterface> TexturedMaterial;
	UPROPERTY()
	TArray<TObjectPtr<UTexture2D>> Textures;
	UPROPERTY()
	TObjectPtr<UProceduralMeshComponent> Surface;
	/** Flat pictures on the walls: the distance figures (and, later, the signs). */
	UPROPERTY()
	TObjectPtr<UProceduralMeshComponent> Signs;
	int32 SignCount = 0;
	/** The stands' generated faces: risers, banks, roofs. */
	UPROPERTY()
	TObjectPtr<UProceduralMeshComponent> Bowl;
	int32 BowlSections = 0, BowlFaces = 0;
	struct FBbBatch
	{
		UStaticMesh* Mesh;
		FBbPaint Paint;
		bool bShadow;
		TArray<FTransform> At;
	};
	TArray<FBbBatch> Batches;
};
