// The one place sim coordinates become Unreal coordinates (PLAN §5, CLAUDE.md "Units and
// frames"). Sim: right-handed, metres, +X first base, +Y center field, +Z up, origin at the
// rear point of home plate. Unreal: left-handed, centimetres, X forward, Z up.
// ue = 100 × (y, x, z). Swapping X and Y is what changes handedness.
#pragma once

#include "CoreMinimal.h"

namespace BbFrameConv
{
	inline FVector ToUE(double X, double Y, double Z)
	{
		return FVector(100.0 * Y, 100.0 * X, 100.0 * Z);
	}
	inline FVector ToUE(const float P[3])
	{
		return ToUE(P[0], P[1], P[2]);
	}
	/** Sim yaw is measured from +Y toward +X; Unreal yaw from +X toward +Y. Same number. */
	inline FRotator YawToUE(float YawRad)
	{
		return FRotator(0.0, FMath::RadiansToDegrees(YawRad), 0.0);
	}
}
