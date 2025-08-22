// Input RNG data
StructuredBuffer<uint> rngValues : register(t0, space0);

// Input particle positions, packed 16 bits X/Y
StructuredBuffer<uint> initialParticlePositions : register(t1, space0);

// Input matching particle positions, packed 16 bits X/Y
StructuredBuffer<uint> matchPositions : register(t2, space0);

// Output buffer (just has a few variables written to by any successful matches)
RWStructuredBuffer<uint> outBuffer : register(u0, space1);

// Uniforms
cbuffer uniformBuffer : register(b0, space2)
{
    // Number of particles in the last frame of the dust animation (size of initialParticlePositions)
    uint particleCountLastFrame : packoffset(c0);

    // Number of matching particle positions (size of matchPositions)
    uint matchPositionsCount : packoffset(c0.y);

    // Offset where last frame's RNG starts, relative to the start of a dust animation
    uint lastFrameRngOffset : packoffset(c0.z);
};

// Gets the float value from a 32-bit int RNG value
float rng_to_float(uint value, float range) 
{
    return value * 2.3283064365386963e-10 * range;
}

[numthreads(64, 1, 1)]
void main(uint3 GlobalInvocationID : SV_DispatchThreadID)
{
    // Get RNG position
    uint rngPosition = (uint)(GlobalInvocationID.x);
    uint startRngPosition = rngPosition;
    if (startRngPosition < lastFrameRngOffset)
    {
        // Skip if this is too early in the RNG sequence
        return;
    }

    // Rounding offset as used by Direct3D
    const float ROUNDING_OFFSET = 1.0 / 512.0;

    // Bitflag for tracking whether matching positions have already been matched
    uint matchBitflag = 0;

    // Simulate all particles
    for (uint i = 0; i < particleCountLastFrame; i++)
    {
        uint pos = initialParticlePositions[i];
        float px = (float)(pos >> 16);
        float py = (float)(pos & 0xffff);

        float gravity = rng_to_float(rngValues[rngPosition + (i * 2)], 0.5) + 0.2;
        float hspeed = rng_to_float(rngValues[rngPosition + ((i * 2) + 1)], 4.0) - 2.0;
        px += hspeed * 11.0;
        py -= gravity * 66.0;

        // Round final positions as they are displayed on-screen
        int rpx = (int)round(px - ROUNDING_OFFSET);
        int rpy = (int)round(py - ROUNDING_OFFSET);
        
        // Check if any positions match this particle
        for (uint j = 0; j < matchPositionsCount; j++)
        {
            if ((matchBitflag & ((uint)1 << j)) == 0)
            {
                uint mpos = matchPositions[j];
                int mpx = (int)(mpos >> 16);
                int mpy = (int)(mpos & 0xffff);

                if (rpx == mpx && rpy == mpy)
                {
                    matchBitflag |= ((uint)1 << j);
                    break;
                }
            }
        }
    }

    // Check whether all positions have been matched
    for (uint j = 0; j < matchPositionsCount; j++)
    {
        if ((matchBitflag & ((uint)1 << j)) == 0)
        {
            return;
        }
    }

    // If we matched, increment the number of matches, and track the position
    InterlockedAdd(outBuffer[0], 1);
    outBuffer[1] = startRngPosition - lastFrameRngOffset;
}
