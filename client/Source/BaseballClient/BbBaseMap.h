// The little diamond in the score display, driven by the play's own timeline (owner, 2026-11-21):
// which bases hold a runner at one moment of the playback, read from where the runners are in
// the sampled frame. Physics is authoritative: this is a reading of the actors' positions, not a
// second model of the runners.
#pragma once

#include "CoreMinimal.h"
#include "BbSim.h"

namespace BbBaseMap
{
	/** A runner counts as on a base while he is within this distance of it (sim plane, m). The
	 *  bag is 0.38 m square and a runner stands on it or at its corner; a runner who has left
	 *  it is past this within a stride or two. Between bases, nothing is lit. */
	constexpr float OnBaseM = 1.0f;

	/** Bit 0 first, bit 1 second, bit 2 third: the bases with a runner (an actor without the
	 *  fielder bit 0x80) on them in this frame. Park.bases are the three corners of the
	 *  infield square. */
	inline uint8 Occupied(const BbFrame& F, const BbPark& Park)
	{
		uint8 Bits = 0;
		for (uint32 I = 0; I < F.n_actors && I < BB_MAX_ACTORS; ++I)
		{
			if (F.actors[I].agent & 0x80)
			{
				continue;
			}
			for (int32 B = 0; B < 3; ++B)
			{
				const float DX = F.actors[I].pos[0] - Park.bases[B][0];
				const float DY = F.actors[I].pos[1] - Park.bases[B][1];
				if (DX * DX + DY * DY <= OnBaseM * OnBaseM)
				{
					Bits |= (uint8)(1 << B);
				}
			}
		}
		return Bits;
	}
}
