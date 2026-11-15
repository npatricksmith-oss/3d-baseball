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
	inline const FLinearColor GrassLight(0.070f, 0.235f, 0.050f);
	inline const FLinearColor GrassDark(0.056f, 0.200f, 0.042f);
	inline const FLinearColor Dirt(0.400f, 0.240f, 0.130f);
	inline const FLinearColor Track(0.330f, 0.185f, 0.105f);
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
