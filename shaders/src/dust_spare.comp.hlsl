// Input RNG data
StructuredBuffer<uint> rngValues : register(t0, space0);

// Input matching particle positions, packed 16 bits X/Y
StructuredBuffer<uint> matchPositions : register(t1, space0);

// Output buffer (just has a few variables written to by any successful matches)
RWStructuredBuffer<uint> outBuffer : register(u0, space1);

// Uniforms
cbuffer uniformBuffer : register(b0, space2)
{
    // Packed 16 bits X/Y position of the animation
    uint animPosition : packoffset(c0);

    // Packed 16 bits X/Y width and height of the animation
    uint animSize : packoffset(c0.y);

    // Number of matching particle positions (size of matchPositions)
    uint matchPositionsCount : packoffset(c0.z);
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

    // Unpack position and size
    float animX = (float)(animPosition >> 16);
    float animY = (float)(animPosition & 0xffff);
    float animWidth = (float)(animSize >> 16);
    float animHeight = (float)(animSize & 0xffff);

    // Simulate particles (on frames 2, 3, 4, 5, 6)
    const uint PARTICLE_COUNT = 14;
    const uint FRAME_COUNT = 6;
    static float frameDistances[FRAME_COUNT] =
    {
        7.2,
        7.2 + 6.4,
        7.2 + 6.4 + 5.6,
        7.2 + 6.4 + 5.6 + 4.8,
        7.2 + 6.4 + 5.6 + 4.8 + 4.0,
        7.2 + 6.4 + 5.6 + 4.8 + 4.0 + 3.2
    };
    const float GML_EPSILON = 0.00001;
    const float TOLERANCE = 3.0;
    uint matchIndex = FRAME_COUNT;
    for (uint frameIndex = 0; frameIndex < FRAME_COUNT; frameIndex++)
    {
        float frameDistance = frameDistances[frameIndex];

        // Bitflag for tracking whether matching positions have already been matched
        uint matchBitflag = 0;

        for (uint i = 0; i < PARTICLE_COUNT; i++)
        {
            float particleY = rng_to_float(rngValues[rngPosition + (i * 3)], animHeight * 0.5) + (animWidth * 0.25) + animY;
            float particleX = rng_to_float(rngValues[rngPosition + (i * 3) + 1], animWidth * 0.5) + (animWidth * 0.25) + animX;

            float rightSide = (particleX - animX) / (animWidth * 0.5);
            float topSide = (particleY - animY) / (animHeight * 0.5);

            float particleHalfSize = rng_to_float(rngValues[rngPosition + (i * 3) + 2], 8.0) + (0.7 * 8.0);
            particleX += particleHalfSize;
            particleX -= 8.0;
            particleY += particleHalfSize;
            particleY -= 8.0;

            float direction = -rng_to_float(rngValues[rngPosition + (PARTICLE_COUNT * 3) + 38 + i], 360.0);
            if (direction < 0.0)
            {
                direction += 360.0;
            }

            // Let's just hope the compiler is able to be a bit smarter about this.
            if (rightSide <= (0.75 - GML_EPSILON))
            {
                direction = -180.0 + 360.0;
            }
            if (rightSide >= (1.25 + GML_EPSILON))
            {
                direction = 0.0;
            }
            if (topSide >= (1.25 + GML_EPSILON) && rightSide >= (1.25 + GML_EPSILON))
            {
                direction = -45.0 + 360.0;
            }
            if (topSide >= (1.25 + GML_EPSILON) && rightSide >= (0.75 + GML_EPSILON) && rightSide <= (1.25 - GML_EPSILON))
            {
                direction = -90.0 + 360.0;
            }
            if (topSide >= (1.25 + GML_EPSILON) && rightSide <= (0.75 - GML_EPSILON))
            {
                direction = -135.0 + 360.0;
            }
            if (topSide <= (0.75 - GML_EPSILON) && rightSide >= (1.25 + GML_EPSILON))
            {
                direction = -315.0 + 360.0;
            }
            if (topSide <= (0.75 - GML_EPSILON) && rightSide >= (0.75 + GML_EPSILON) && rightSide <= (1.25 - GML_EPSILON))
            {
                direction = -270.0 + 360.0;
            }
            if (topSide <= (0.75 - GML_EPSILON) && rightSide <= (0.75 - GML_EPSILON))
            {
                direction = -235.0 + 360.0;
            }

            float angleRadians = (direction * 3.1415927) / 180.0;
            particleX += frameDistance * cos(angleRadians);
            particleY -= frameDistance * sin(angleRadians);

            // Check if any positions match this particle
            for (uint j = 0; j < matchPositionsCount; j++)
            {
                if ((matchBitflag & ((uint)1 << j)) == 0)
                {
                    uint mpos = matchPositions[j];
                    float mpx = (float)(int)(mpos >> 16);
                    float mpy = (float)(int)(mpos & 0xffff);

                    if (abs(particleX - mpx) < TOLERANCE && abs(particleY - mpy) < TOLERANCE)
                    {
                        matchBitflag |= ((uint)1 << j);
                        break;
                    }
                }
            }
        }

        // Check whether all positions have been matched
        bool anyNonMatch = false;
        for (uint j = 0; j < matchPositionsCount; j++)
        {
            if ((matchBitflag & ((uint)1 << j)) == 0)
            {
                anyNonMatch = true;
            }
        }

        // Found a match!
        if (!anyNonMatch)
        {
            matchIndex = frameIndex;
            break;
        }
    }

    if (matchIndex == FRAME_COUNT)
    {
        return;
    }

    // If we matched, increment the number of matches, and track the position
    InterlockedAdd(outBuffer[0], 1);
    outBuffer[1] = rngPosition;
    outBuffer[2] = matchIndex + 1;
}
