// The little diamond in the score display, driven by the play's own timeline (owner, 2026-11-21):
// each runner is a dot that moves on the diamond with his real position in the sampled frame,
// and a base is lit while a runner is at it. Physics is authoritative: this only reads the
// actors' positions and projects them; there is no second model of the runners.
#pragma once

#include "CoreMinimal.h"
#include "BbSim.h"

namespace BbBaseMap
{
	/** A base is lit for a runner from when he reaches it, ArriveS before it along the path
	 *  (about 1.5 m: he is on the bag or the next stride puts him there), until he has committed
	 *  to the next one, HoldS past it (half way along the side, about 13.7 m). That keeps it lit
	 *  through his lead (a long one too), an overrun, and his return to the bag; it goes out
	 *  half way to the next base, and the next base lights as he reaches it. A runner who rounds
	 *  a base and runs on lights it as he passes (he has reached it), and not again. */
	constexpr float ArriveS = 0.055f;
	constexpr float HoldS = 0.5f;
	/** A runner within this distance of the plate (m) is not drawn: the batter in his box,
	 *  and a runner who has scored. The dot appears when the batter leaves the box. */
	constexpr float HomeHideM = 1.8f;

	/** A sim-frame point on the diamond, in units where home is (0, 0), first base (1, 1),
	 *  second (0, 2), third (-1, 1): x toward first base, y toward second, the same orientation
	 *  as the picture (second at the top). Affine, so a point halfway along a base path maps
	 *  halfway; built from the park's own base corners. */
	inline FVector2D ToDiamond(float X, float Y, const BbPark& Park)
	{
		// pos = a * first + b * third; the diamond is (a - b, a + b).
		const double FX = Park.bases[0][0], FY = Park.bases[0][1], TX = Park.bases[2][0], TY = Park.bases[2][1];
		const double Det = FX * TY - TX * FY;
		const double A = (X * TY - TX * Y) / Det;
		const double B = (FX * Y - X * FY) / Det;
		return FVector2D(A - B, A + B);
	}

	/** A runner's progress along the base path, in bases: 0 at the plate, 1 at first, 2 at
	 *  second, 3 at third, 4 back at the plate. It is his place projected onto the nearest side of
	 *  the diamond (clamped to the side's ends, so an overrun past a bag, which lies on the line
	 *  of the side he came by, is still that bag). A function of where he is now, so it needs no
	 *  history and is right when paused, scrubbed or reopened. */
	inline float PathProgress(float X, float Y, const BbPark& Park)
	{
		const FVector2D U = ToDiamond(X, Y, Park);
		const FVector2D Corner[5] = { FVector2D(0, 0), FVector2D(1, 1), FVector2D(0, 2), FVector2D(-1, 1), FVector2D(0, 0) };
		float Best = 0.f;
		double BestD = TNumericLimits<double>::Max();
		for (int32 Side = 0; Side < 4; ++Side)
		{
			const FVector2D A = Corner[Side], D = Corner[Side + 1] - A;
			const double T = FMath::Clamp(FVector2D::DotProduct(U - A, D) / FVector2D::DotProduct(D, D), 0.0, 1.0);
			const double Dist = FVector2D::Distance(U, A + D * T);
			if (Dist < BestD - 1e-9)
			{
				BestD = Dist;
				Best = (float)(Side + T);
			}
		}
		return Best;
	}

	/** One runner as the map draws him. */
	struct FDot
	{
		uint16 Agent = 0;
		FVector2D At = FVector2D::ZeroVector;
	};

	/** A foul ball, as the sim rules it: its summary is exactly "Foul" when the plate appearance
	 *  goes on after a batted ball ruled foul (the pitch call is only a guess from where the
	 *  ball first lands). A structured flag from bb_ffi would replace this text match
	 *  (docs/hud.md); the director's foul-ball signal reads the summary the same way. */
	inline bool IsFoul(const BbPlayInfo& Info)
	{
		return FCStringAnsi::Strcmp(Info.summary, "Foul") == 0;
	}

	/** What the map shows at one moment of a play. */
	struct FMap
	{
		/** Bit 0 first, bit 1 second, bit 2 third: the bases a runner has reached and not left. */
		uint8 Lit = 0;
		TArray<FDot> Dots;
	};

	/** The map at play time T from the sampled frame F, for a play whose summary is Info and
	 *  which began with the runners BasesBefore (bit 0 first ... bit 2 third) and OutsBefore outs.
	 *  Every actor without the fielder bit (0x80) is a runner, but not drawn (nor does he light a
	 *  base) when:
	 *  - he is the batter still in his box, or a runner who has scored (within HomeHideM of the
	 *    plate);
	 *  - the umpire has called him out (BbCall.runner, from its t_s): the timeline leaves him
	 *    standing where he was put out;
	 *  - the third out has been called: the half inning is over, and the sim's result clears the
	 *    bases.
	 *  A foul ball (IsFoul) is the exception to all of it: the timeline has the batter and the
	 *  runners break and stops with them off their bases, but by the rules they go back, so the
	 *  map keeps the runners where they were before the pitch: the bases lit, a dot on each. */
	inline void Read(const BbFrame& F, const BbPark& Park, const BbPlayInfo& Info, uint8 BasesBefore, int32 OutsBefore, float T, FMap& Out)
	{
		Out.Lit = 0;
		Out.Dots.Reset();
		int32 Called = 0;
		while (Called < (int32)Info.n_calls && Called < BB_MAX_CALLS && T >= Info.calls[Called].t_s)
		{
			++Called;
		}
		if (OutsBefore + Called >= 3)
		{
			return;
		}
		if (IsFoul(Info))
		{
			for (int32 B = 0; B < 3; ++B)
			{
				if ((BasesBefore >> B) & 1)
				{
					FDot D;
					D.At = ToDiamond(Park.bases[B][0], Park.bases[B][1], Park);
					Out.Dots.Add(D);
					Out.Lit |= (uint8)(1 << B);
				}
			}
			return;
		}
		for (uint32 I = 0; I < F.n_actors && I < BB_MAX_ACTORS; ++I)
		{
			const uint16 Agent = F.actors[I].agent;
			const float* P = F.actors[I].pos;
			if ((Agent & 0x80) || P[0] * P[0] + P[1] * P[1] <= HomeHideM * HomeHideM)
			{
				continue;
			}
			bool bOut = false;
			for (int32 C = 0; C < Called; ++C)
			{
				bOut |= Info.calls[C].runner == Agent;
			}
			if (bOut)
			{
				continue;
			}
			FDot D;
			D.Agent = Agent;
			D.At = ToDiamond(P[0], P[1], Park);
			Out.Dots.Add(D);
			const float Along = PathProgress(P[0], P[1], Park);
			for (int32 B = 1; B <= 3; ++B)
			{
				if (Along >= B - ArriveS && Along <= B + HoldS)
				{
					Out.Lit |= (uint8)(1 << (B - 1));
				}
			}
		}
	}
}
