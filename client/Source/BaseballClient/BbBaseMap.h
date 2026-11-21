// The little diamond in the score display, driven by the play's own timeline (owner, 2026-11-21):
// each runner is a dot that moves on the diamond with his real position in the sampled frame,
// and a base is lit while a runner is at it. Physics is authoritative: this only reads the
// actors' positions and projects them; there is no second model of the runners.
#pragma once

#include "CoreMinimal.h"
#include "BbSim.h"

namespace BbBaseMap
{
	/** A base is lit while a runner is within this distance of it (sim plane, m): he stands on
	 *  the bag, or takes his lead (about 5 m), or overruns it. Between bases nothing is lit and
	 *  only his dot shows. Leaving the zone for the next base puts the light out. */
	constexpr float LitZoneM = 7.0f;
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

	/** One runner as the map draws him. */
	struct FDot
	{
		uint16 Agent = 0;
		FVector2D At = FVector2D::ZeroVector;
	};

	/** A foul ball: pitch_call 4 or 5, or a batted ball the sim's own summary calls "Foul" (a
	 *  ball in play on the pitch call, ruled foul by where it came down; the director reads
	 *  the summary the same way). */
	inline bool IsFoul(const BbPlayInfo& Info)
	{
		return Info.pitch_call == 4 || Info.pitch_call == 5 || FCStringAnsi::Strcmp(Info.summary, "Foul") == 0
			|| (FCStringAnsi::Strlen(Info.summary) >= 4 && FCStringAnsi::Strcmp(Info.summary + FCStringAnsi::Strlen(Info.summary) - 4, "Foul") == 0);
	}

	/** What the map shows at one moment of a play. */
	struct FMap
	{
		/** Bit 0 first, bit 1 second, bit 2 third: the bases with a runner in their zone. */
		uint8 Lit = 0;
		TArray<FDot> Dots;
	};

	/** The map at play time T from the sampled frame F, for a play whose summary is Info and
	 *  which began with the runners BasesBefore (bit 0 first ... bit 2 third) and OutsBefore outs. Every actor without the fielder bit (0x80) is a runner,
	 *  but not drawn (nor does he light a base):
	 *  - the batter still in his box and a runner who has scored (within HomeHideM of the plate);
	 *  - a runner the umpire has called out (BbCall.runner, from its t_s: the timeline leaves him
	 *    standing where he was put out);
	 *  A foul ball (IsFoul) is the exception: the timeline has the batter and the
	 *  runners break and stops with them off their bases, but by the rules they go back, so the
	 *  map keeps the runners where they were before the pitch: the bases lit, a dot on each.
	 *  - everyone once the third out has been called: the half inning is over, and the sim's
	 *    result clears the bases. */
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
			for (int32 B = 0; B < 3; ++B)
			{
				const float DX = P[0] - Park.bases[B][0];
				const float DY = P[1] - Park.bases[B][1];
				if (DX * DX + DY * DY <= LitZoneM * LitZoneM)
				{
					Out.Lit |= (uint8)(1 << B);
				}
			}
		}
	}
}
