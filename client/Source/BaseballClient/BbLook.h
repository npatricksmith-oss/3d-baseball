// The park's look, in one place (docs/park-visuals.md, step 2): every surface colour, the
// afternoon sun, the sky's fill, the shadows, and the grade the picture is given. Nothing
// else in the client names a colour for anything that stands still.
#pragma once

#include "CoreMinimal.h"

/** A surface: its colour (linear) and how matt it is (1 = chalk, 0.3 = fresh gloss paint). */
struct FBbPaint
{
	FLinearColor Color;
	float Rough = 0.9f;
};

namespace BbLook
{
	// ---- The palette --------------------------------------------------------------------
	// The ground (painted into the ground's pictures; all of it matt).
	// Grass darkened by 17% and dirt by 20%, then to 32% the same day (owner, 2026-11-16) so the white ball stands out;
	// before: grass (0.070, 0.235, 0.050) and (0.056, 0.200, 0.042), dirt (0.400, 0.240, 0.130),
	// track (0.330, 0.185, 0.105).
	inline const FLinearColor GrassLight(0.058f, 0.195f, 0.042f);
	inline const FLinearColor GrassDark(0.046f, 0.166f, 0.035f);
	inline const FLinearColor Dirt(0.270f, 0.162f, 0.088f);
	inline const FLinearColor Track(0.223f, 0.125f, 0.071f);
	inline const FLinearColor Chalk(0.880f, 0.880f, 0.850f);
	inline const float GroundRough = 0.95f;
	/** The land outside the park. */
	inline const FBbPaint Land{ FLinearColor(0.050f, 0.150f, 0.045f), 0.95f };

	// What stands on it.
	/** Bases, the rubber. */
	inline const FBbPaint Rubber{ FLinearColor(0.880f, 0.880f, 0.850f), 0.7f };
	/** Wall padding: dark green, so a white ball shows against it. */
	inline const FBbPaint WallPad{ FLinearColor(0.020f, 0.085f, 0.055f), 0.75f };
	/** Painted steel: screen posts, rails. */
	inline const FBbPaint Steel{ FLinearColor(0.560f, 0.580f, 0.600f), 0.45f };
	inline const FBbPaint FoulPole{ FLinearColor(0.900f, 0.720f, 0.060f), 0.45f };
	inline const FBbPaint Seats{ FLinearColor(0.045f, 0.110f, 0.230f), 0.55f };
	inline const FBbPaint Concrete{ FLinearColor(0.400f, 0.385f, 0.360f), 0.9f };
	inline const FBbPaint Roof{ FLinearColor(0.110f, 0.130f, 0.140f), 0.6f };
	/** The dark of a dugout's floor. */
	inline const FBbPaint DugoutFloor{ FLinearColor(0.030f, 0.030f, 0.035f), 0.95f };
	/** A dugout's walls and steps, and its bench. */
	inline const FBbPaint DugoutWall{ FLinearColor(0.170f, 0.175f, 0.180f), 0.9f };
	inline const FBbPaint Bench{ FLinearColor(0.330f, 0.200f, 0.100f), 0.7f };
	/** The grass bank beyond the left-field fence, and the benches beyond the right. */
	inline const FBbPaint Berm{ FLinearColor(0.062f, 0.210f, 0.046f), 0.95f };
	inline const FBbPaint Bench2{ FLinearColor(0.600f, 0.620f, 0.640f), 0.4f };
	/** The pressbox: its walls and its band of glass. */
	inline const FBbPaint Pressbox{ FLinearColor(0.560f, 0.540f, 0.500f), 0.85f };
	inline const FBbPaint Glass{ FLinearColor(0.015f, 0.030f, 0.045f), 0.12f };
	/** The lamps on the light towers, unlit. */
	inline const FBbPaint Lamp{ FLinearColor(0.780f, 0.780f, 0.740f), 0.35f };
	/** The line along the top of the outfield wall: over it is a home run. */
	inline const FBbPaint WallCap{ FLinearColor(0.900f, 0.720f, 0.060f), 0.5f };
	/** The distance figures on the wall. */
	inline const FLinearColor MarkerInk(0.880f, 0.880f, 0.850f);
	/** The batter's eye: a dark matt wall behind centre field, for the batter to see the ball against. */
	inline const FBbPaint BatterEye{ FLinearColor(0.010f, 0.040f, 0.026f), 0.97f };
	/** The backstop's net: how dark it is and how much of what is behind it it hides. */
	inline const FLinearColor Net(0.030f, 0.030f, 0.035f);
	inline const float NetOpacity = 0.14f;

	// The crowd (park-visuals step 6): how many came, and what they wear. Attendance is the
	// one number; the places fill by how sought-after they are (low rows near the plate
	// first, the bank last), the same way every time the park is built.
	inline const int32 Attendance = 6200;
	inline const int32 CrowdSeed = 11;
	/** Shirts: mostly mid and dark, one pale in the lot, so a ball in front of the seats shows. */
	inline const FBbPaint Shirts[] = {
		{ FLinearColor(0.420f, 0.030f, 0.030f), 0.85f },
		{ FLinearColor(0.020f, 0.045f, 0.200f), 0.85f },
		{ FLinearColor(0.030f, 0.130f, 0.420f), 0.85f },
		{ FLinearColor(0.040f, 0.200f, 0.075f), 0.85f },
		{ FLinearColor(0.600f, 0.440f, 0.050f), 0.85f },
		{ FLinearColor(0.560f, 0.180f, 0.030f), 0.85f },
		{ FLinearColor(0.260f, 0.265f, 0.280f), 0.85f },
		{ FLinearColor(0.500f, 0.500f, 0.490f), 0.85f },
		{ FLinearColor(0.740f, 0.740f, 0.710f), 0.85f },
		{ FLinearColor(0.020f, 0.020f, 0.025f), 0.85f },
		{ FLinearColor(0.030f, 0.230f, 0.260f), 0.85f },
		{ FLinearColor(0.180f, 0.045f, 0.270f), 0.85f },
		{ FLinearColor(0.560f, 0.230f, 0.280f), 0.85f },
		{ FLinearColor(0.400f, 0.310f, 0.190f), 0.85f },
	};
	inline const FBbPaint Skin[] = {
		{ FLinearColor(0.600f, 0.390f, 0.280f), 0.7f },
		{ FLinearColor(0.440f, 0.265f, 0.170f), 0.7f },
		{ FLinearColor(0.290f, 0.165f, 0.100f), 0.7f },
		{ FLinearColor(0.115f, 0.062f, 0.040f), 0.7f },
	};
	/** What is on the head: hair (four), or a cap (three). */
	inline const FBbPaint Heads[] = {
		{ FLinearColor(0.012f, 0.010f, 0.010f), 0.8f },
		{ FLinearColor(0.085f, 0.048f, 0.024f), 0.8f },
		{ FLinearColor(0.460f, 0.350f, 0.140f), 0.8f },
		{ FLinearColor(0.380f, 0.380f, 0.380f), 0.8f },
		{ FLinearColor(0.020f, 0.045f, 0.200f), 0.85f },
		{ FLinearColor(0.420f, 0.030f, 0.030f), 0.85f },
		{ FLinearColor(0.700f, 0.700f, 0.670f), 0.85f },
	};
	inline const FBbPaint Trousers[] = {
		{ FLinearColor(0.045f, 0.085f, 0.190f), 0.9f },
		{ FLinearColor(0.400f, 0.320f, 0.210f), 0.9f },
		{ FLinearColor(0.030f, 0.030f, 0.035f), 0.9f },
	};

	// The scoreboard: lamps on a dark face, in a dark case on steel legs. The lamps give
	// their own light (BoardGlow times their colour), so the board reads in the shade.
	inline const TCHAR* const ParkName = TEXT("CEDAR HOLLOW PARK");
	inline const FBbPaint BoardCase{ FLinearColor(0.035f, 0.050f, 0.045f), 0.7f };
	inline const FLinearColor BoardBack(0.004f, 0.005f, 0.005f);
	inline const FLinearColor BoardInk(1.000f, 0.860f, 0.560f);
	inline const FLinearColor BoardDim(0.330f, 0.300f, 0.230f);
	inline const FLinearColor BoardTitle(1.000f, 0.560f, 0.080f);
	inline const FLinearColor LampBall(0.080f, 0.900f, 0.140f);
	inline const FLinearColor LampStrike(1.000f, 0.100f, 0.050f);
	inline const FLinearColor LampOut(1.000f, 0.100f, 0.050f);
	inline const FLinearColor LampOff(0.030f, 0.030f, 0.030f);
	inline const float BoardGlow = 5.f;

	// The signs on the fence: invented names, none of them a real firm's. Dark grounds, so a
	// white ball still shows in front of them.
	struct FBbSponsor
	{
		const TCHAR* Top;
		/** A second line, or null. */
		const TCHAR* Bottom;
		FLinearColor Back, Ink;
	};
	inline const FBbSponsor Sponsors[] = {
		{ TEXT("HARBOR"), TEXT("SAVINGS BANK"), FLinearColor(0.015f, 0.030f, 0.120f), FLinearColor(0.800f, 0.760f, 0.600f) },
		{ TEXT("OAK STREET"), TEXT("DINER"), FLinearColor(0.180f, 0.016f, 0.014f), FLinearColor(0.820f, 0.780f, 0.620f) },
		{ TEXT("NORTHSIDE"), TEXT("AUTO REPAIR"), FLinearColor(0.030f, 0.032f, 0.036f), FLinearColor(0.850f, 0.640f, 0.060f) },
		{ TEXT("RIVERBEND"), TEXT("DAIRY"), FLinearColor(0.010f, 0.085f, 0.095f), FLinearColor(0.780f, 0.800f, 0.760f) },
		{ TEXT("TALLY'S TIRES"), nullptr, FLinearColor(0.020f, 0.020f, 0.022f), FLinearColor(0.850f, 0.300f, 0.040f) },
		{ TEXT("LANTERN"), TEXT("BOOKS"), FLinearColor(0.085f, 0.016f, 0.080f), FLinearColor(0.820f, 0.760f, 0.600f) },
		{ TEXT("STONY CREEK"), TEXT("FEED & SEED"), FLinearColor(0.110f, 0.050f, 0.018f), FLinearColor(0.820f, 0.780f, 0.620f) },
		{ TEXT("BLUE HERON"), TEXT("COFFEE"), FLinearColor(0.030f, 0.060f, 0.110f), FLinearColor(0.700f, 0.800f, 0.860f) },
		{ TEXT("UNION"), TEXT("PRINT CO."), FLinearColor(0.045f, 0.055f, 0.016f), FLinearColor(0.850f, 0.700f, 0.100f) },
		{ TEXT("KESTREL"), TEXT("RADIO 1040"), FLinearColor(0.130f, 0.014f, 0.040f), FLinearColor(0.800f, 0.780f, 0.720f) },
	};

	// Beyond the park: two lines of hills, the far one paler and bluer, and trees.
	inline const FBbPaint HillNear{ FLinearColor(0.040f, 0.115f, 0.045f), 0.95f };
	inline const FBbPaint HillFar{ FLinearColor(0.075f, 0.135f, 0.130f), 0.95f };
	inline const FBbPaint TreeCrown[3] = {
		{ FLinearColor(0.030f, 0.120f, 0.030f), 0.9f },
		{ FLinearColor(0.045f, 0.150f, 0.035f), 0.9f },
		{ FLinearColor(0.060f, 0.135f, 0.025f), 0.9f },
	};
	inline const FBbPaint Conifer{ FLinearColor(0.016f, 0.075f, 0.035f), 0.9f };
	inline const FBbPaint TreeTrunk{ FLinearColor(0.085f, 0.055f, 0.035f), 0.9f };
	/** The same wood every time the park is built. */
	inline const int32 TreeSeed = 5;

	// Flags: plain fields with a band across, on white poles. They fly toward this bearing
	// (as the sun's: from +Y round toward +X); a look only, the sim has no wind.
	inline const FBbPaint FlagPole{ FLinearColor(0.800f, 0.800f, 0.780f), 0.4f };
	inline const FBbPaint Flags[3] = {
		{ FLinearColor(0.450f, 0.020f, 0.020f), 0.8f },
		{ FLinearColor(0.020f, 0.060f, 0.300f), 0.8f },
		{ FLinearColor(0.750f, 0.560f, 0.030f), 0.8f },
	};
	inline const FBbPaint FlagBand{ FLinearColor(0.850f, 0.850f, 0.820f), 0.8f };
	inline const float FlagBearingDeg = 60.f;

	// ---- The players (docs/player-visuals.md) ---------------------------------------------
	// Two invented clubs: the home side in white, the visitors in grey, each with a trim
	// colour for its lettering, number, socks and cap. Neither white nor grey is as bright
	// as the ball, so the ball still shows in front of a uniform.
	struct FBbClub
	{
		const TCHAR* Place;
		const TCHAR* Name;
		/** Across the chest: the name at home, the place on the road. */
		const TCHAR* Chest;
		FBbPaint Jersey, Pants, Socks, Cap, Bill;
		/** Lettering, number, collar. */
		FLinearColor Trim;
	};
	/** [0] the visitors, [1] the home side (the sim's sides). */
	inline const FBbClub Clubs[2] = {
		{ TEXT("STONEBRIDGE"), TEXT("COLLIERS"), TEXT("STONEBRIDGE"),
			{ FLinearColor(0.300f, 0.310f, 0.325f), 0.85f }, { FLinearColor(0.300f, 0.310f, 0.325f), 0.85f },
			{ FLinearColor(0.200f, 0.012f, 0.010f), 0.85f }, { FLinearColor(0.022f, 0.022f, 0.026f), 0.8f },
			{ FLinearColor(0.200f, 0.012f, 0.010f), 0.7f }, FLinearColor(0.200f, 0.012f, 0.010f) },
		{ TEXT("CEDAR HOLLOW"), TEXT("LARKS"), TEXT("LARKS"),
			{ FLinearColor(0.720f, 0.720f, 0.690f), 0.85f }, { FLinearColor(0.720f, 0.720f, 0.690f), 0.85f },
			{ FLinearColor(0.006f, 0.014f, 0.075f), 0.85f }, { FLinearColor(0.006f, 0.014f, 0.075f), 0.8f },
			{ FLinearColor(0.006f, 0.014f, 0.075f), 0.7f }, FLinearColor(0.006f, 0.014f, 0.075f) },
	};
	// The umpires (player-visuals step 7): a pale blue shirt, charcoal trousers to the shoe (the
	// "sock" is the trousers' colour), a black cap. No lettering. Dressed as a third side.
	inline const FBbClub Umpires = { TEXT(""), TEXT(""), TEXT(""),
		{ FLinearColor(0.230f, 0.360f, 0.520f), 0.85f }, { FLinearColor(0.040f, 0.042f, 0.048f), 0.85f },
		{ FLinearColor(0.040f, 0.042f, 0.048f), 0.85f }, { FLinearColor(0.012f, 0.012f, 0.014f), 0.8f },
		{ FLinearColor(0.012f, 0.012f, 0.014f), 0.7f }, FLinearColor(0.180f, 0.290f, 0.430f) };
	inline const int32 UmpireSide = 2;
	/** The uniform of a side: 0 the visitors, 1 the home side, 2 the umpires. */
	inline const FBbClub& Club(int32 Side) { return Side == UmpireSide ? Umpires : Clubs[Side & 1]; }
	// ---- The game graphics (docs/hud.md) ------------------------------------------------
	// Given as the sRGB a paint program would show. The clubs' panel colours are their trims,
	// lifted: the uniforms' navy and dark red are too near black to read on a dark panel.
	inline const FLinearColor HudPanel = FLinearColor::FromSRGBColor(FColor(16, 20, 28, 240));
	inline const FLinearColor HudPanelLight = FLinearColor::FromSRGBColor(FColor(36, 42, 54, 240));
	inline const FLinearColor HudText = FLinearColor::FromSRGBColor(FColor(246, 246, 242));
	inline const FLinearColor HudDim = FLinearColor::FromSRGBColor(FColor(150, 158, 172));
	/** Lit bases, the replay tag, the home run's edge. */
	inline const FLinearColor HudGold = FLinearColor::FromSRGBColor(FColor(244, 196, 48));
	/** An empty base, an out not yet made. */
	inline const FLinearColor HudOff = FLinearColor::FromSRGBColor(FColor(66, 74, 90));
	/** The strike-zone box's marks: a ball, a ball put in play (a called strike is HudClose,
	 *  a swing or a foul HudGold). */
	inline const FLinearColor HudBall = FLinearColor::FromSRGBColor(FColor(84, 196, 120));
	inline const FLinearColor HudInPlay = FLinearColor::FromSRGBColor(FColor(92, 150, 250));
	/** A bang-bang out. */
	inline const FLinearColor HudClose = FLinearColor::FromSRGBColor(FColor(222, 64, 52));
	/** [0] the visitors, [1] the home side, as in Clubs. */
	inline const FLinearColor HudClub[2] = { FLinearColor::FromSRGBColor(FColor(150, 30, 28)), FLinearColor::FromSRGBColor(FColor(30, 56, 132)) };

	// The benches (step 7): reserves and a manager in each dugout besides the nine.
	inline const int32 BenchReserves = 4;
	inline const FBbPaint Shoe{ FLinearColor(0.012f, 0.012f, 0.014f), 0.5f };
	inline const FBbPaint Belt{ FLinearColor(0.015f, 0.015f, 0.018f), 0.45f };
	// The kit (player-visuals step 2). A helmet is the cap's colour with a shine; the
	// catcher's chest protector and shin guards are the club's trim.
	inline const float HelmetRough = 0.22f, GearRough = 0.6f;
	inline const FBbPaint Mask{ FLinearColor(0.014f, 0.014f, 0.016f), 0.45f };
	inline const FBbPaint Leather{ FLinearColor(0.150f, 0.062f, 0.022f), 0.7f };
	inline const FBbPaint Wood{ FLinearColor(0.520f, 0.330f, 0.150f), 0.45f };
	/** A glove, cm: half its thickness (palm to back), half its width, half its length. Larger
	 *  than life (a real one is about 30 long), as the ball and the bat are drawn. */
	inline const float GloveThickCm = 4.0f, GloveWideCm = 9.0f, GloveLongCm = 13.0f;
	/** The catcher's mitt against a fielder's glove. */
	inline const float MittScale = 1.18f;
	/** The bat is drawn this much thicker than real so that the swing reads from the stands. */
	inline const float BatThick = 1.35f;
	/** Players' skin tones are the crowd's (Skin); their hair is the crowd's first three Heads. */
	inline const int32 HairTones = 3;
	// Standing height and build are drawn by player ID from these ranges, with values
	// clustered near the middle.
	inline const float HeightMinCm = 177.f, HeightMaxCm = 197.f;
	/** How thick the trunk and limbs are, against the middle build. */
	inline const float BuildMin = 0.92f, BuildMax = 1.14f;

	// ---- The light: a clear mid-afternoon ------------------------------------------------
	// Where the sun stands, in the sim frame: the bearing on the ground from +Y (centre
	// field) round toward +X (first base), and the height above the horizon. Over the
	// third-base side and a little to the outfield, so both the camera behind home and the
	// one in centre field see lit faces and shadows that fall across the picture.
	inline const float SunBearingDeg = 282.f;
	inline const float SunHeightDeg = 42.f;
	inline const float SunLux = 7.5f;
	inline const FLinearColor SunColor(1.000f, 0.955f, 0.880f);
	/** The sky's fill, and what comes back up off the grass. */
	inline const float SkyFill = 2.3f;
	inline const FLinearColor GroundBounce(0.100f, 0.160f, 0.080f);

	// Shadows: four cascades reaching past the far wall from the centre-field camera.
	inline const float ShadowReachM = 320.f;
	inline const int32 ShadowCascades = 4;
	inline const float ShadowCascadeExponent = 2.6f;

	// ---- The picture ---------------------------------------------------------------------
	/** Fixed exposure, in stops from the engine's default. */
	inline const float ExposureStops = -0.75f;
	inline const float AoIntensity = 0.6f;
	inline const float AoRadiusCm = 90.f;
	inline const float Saturation = 1.07f;
	inline const float Contrast = 1.05f;
	inline const float BloomIntensity = 0.25f;
	inline const float Vignette = 0.2f;
}
