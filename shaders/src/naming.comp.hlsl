// Input RNG seeds
StructuredBuffer<uint> rngSeeds : register(t0, space0);

// Output buffer (just has a few variables written to by any successful matches)
RWStructuredBuffer<uint> outBuffer : register(u0, space1);

// Uniforms
cbuffer uniformBuffer : register(b0, space2)
{
    // Flags used for RNG initialization:
    // Bit 0 is whether 15-bit
    // Bit 1 is whether signed
    // Bit 2 is whether using old random polynomial
    uint randomFlags : packoffset(c0);

    // Range of RNG values to search within each seed
    uint searchRange : packoffset(c0.y);
    
    // Input matching pixels: four 32-bit numbers, of which only the first 104 bits are used (lower bits of the fourth number)
    uint match1 : packoffset(c0.z);
    uint match2 : packoffset(c0.w);
    uint match3 : packoffset(c1);
    uint match4 : packoffset(c1.y);
};

[numthreads(64, 1, 1)]
void main(uint3 GlobalInvocationID : SV_DispatchThreadID)
{
    // Get RNG seed
    uint rngSeedIndex = (uint)(GlobalInvocationID.x);
    uint rngSeed = rngSeeds[rngSeedIndex];

    // Initialize RNG state
    uint rngPoly = ((randomFlags & ((uint)1 << 2)) != 0) ? 0xda442d20 : 0xda442d24;
    uint rngIndex = 0;
    uint rngState[16] = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
    if ((randomFlags & ((uint)1 << 0)) != 0)
    {
        // Unsigned 15-bit
        uint tempSeed = rngSeed;
        for (uint i = 0; i < 16; i++)
        {
            tempSeed = (((tempSeed * 0x343fd) + 0x269ec3) >> 16) & 0x7fff;
            rngState[i] = tempSeed;
        }
    } 
    else if ((randomFlags & ((uint)1 << 2)) != 0)
    {
        // Signed 16-bit
        int tempSeed = (int)rngSeed;
        for (uint i = 0; i < 16; i++)
        {
            tempSeed = (((tempSeed * 0x343fd) + 0x269ec3) >> 16) & 0x7fffffff;
            rngState[i] = (uint)tempSeed;
        }
    } 
    else
    {
        // Unsigned and signed 16-bit
        int tempSeed = (int)rngSeed;
        tempSeed = (((tempSeed * 0x343fd) + 0x269ec3) >> 16) & 0x7fffffff;
        rngState[0] = (uint)tempSeed;
        for (uint i = 1; i < 16; i++)
        {
            tempSeed = (int)(((uint)((tempSeed * 0x343fd) + 0x269ec3)) >> 16);
            rngState[i] = (uint)tempSeed;
        }
    }

    // Search over entire range. Keep a buffer of four integers, tracking the last 104 pixels,
    // for both "actual" (> 0.25) and "guaranteed" (> 0.252)
    uint actual1 = 0;
    uint actual2 = 0;
    uint actual3 = 0;
    uint actual4 = 0;
    uint guaranteed1 = 0;
    uint guaranteed2 = 0;
    uint guaranteed3 = 0;
    uint guaranteed4 = 0;
    for (uint i = 0; i < searchRange; i += 2)
    {
        // Shift buffers by 2
        actual4 <<= 2;
        guaranteed4 <<= 2;
        actual4 |= (actual3 & ((uint)3 << 30)) >> 30;
        guaranteed4 |= (guaranteed3 & ((uint)3 << 30)) >> 30;
        actual3 <<= 2;
        guaranteed3 <<= 2;
        actual3 |= (actual2 & ((uint)3 << 30)) >> 30;
        guaranteed3 |= (guaranteed2 & ((uint)3 << 30)) >> 30;
        actual2 <<= 2;
        guaranteed2 <<= 2;
        actual2 |= (actual1 & ((uint)3 << 30)) >> 30;
        guaranteed2 |= (guaranteed1 & ((uint)3 << 30)) >> 30;
        actual1 <<= 2;
        guaranteed1 <<= 2;

        // Advance random state, for the vertical position
        uint a = rngState[rngIndex];
        uint b = rngState[(rngIndex + 13) & 15];
        uint c = a ^ b ^ (a << 16) ^ (b << 15);
        b = rngState[(rngIndex + 9) & 15];
        b ^= (b >> 11);
        a = c ^ b;
        rngState[rngIndex] = a;
        uint d = a ^ ((a << 5) & rngPoly);
        rngIndex = (rngIndex + 15) & 15;
        a = rngState[rngIndex];
        uint currentValue = rngState[rngIndex] = a ^ c ^ d ^ (a << 2) ^ (c << 18) ^ (b << 28);

        // Add current value to buffers
        actual1 |= (currentValue >= 2147483647) ? 2 : 0;
        guaranteed1 |= (currentValue >= 2164663517) ? 2 : 0;

        // Advance random state, for the horizontal position
        a = currentValue;
        b = rngState[(rngIndex + 13) & 15];
        c = a ^ b ^ (a << 16) ^ (b << 15);
        b = rngState[(rngIndex + 9) & 15];
        b ^= (b >> 11);
        a = c ^ b;
        rngState[rngIndex] = a;
        d = a ^ ((a << 5) & rngPoly);
        rngIndex = (rngIndex + 15) & 15;
        a = rngState[rngIndex];
        currentValue = rngState[rngIndex] = a ^ c ^ d ^ (a << 2) ^ (c << 18) ^ (b << 28);

        // Add current value to buffer
        actual1 |= (currentValue >= 2147483647) ? 1 : 0;
        guaranteed1 |= (currentValue >= 2164663517) ? 1 : 0;

        // Check actual buffer against match buffer.
        // All 1s in the match buffer *must* be present in the actual buffer.
        // All 0s in the actual buffer *must* not be present in the match buffer.
        // All 1s in the guaranteed buffer *must* be present in the match buffer.
        if ((actual1 & match1) == match1 && (~actual1 & match1) == 0 && (guaranteed1 & match1) == guaranteed1 &&
            (actual2 & match2) == match2 && (~actual2 & match2) == 0 && (guaranteed2 & match2) == guaranteed2 &&
            (actual3 & match3) == match3 && (~actual3 & match3) == 0 && (guaranteed3 & match3) == guaranteed3 &&
            ((actual4 & 0xff) & match4) == match4 && (~(actual4 & 0xff) & match4) == 0 && ((guaranteed4 & 0xff) & match4) == (guaranteed4 & 0xff))
        {
            // If we matched, increment the number of matches, and track the seed/position
            InterlockedAdd(outBuffer[0], 1);
            outBuffer[1] = rngSeed;
            outBuffer[2] = i + 2;
            break;
        }
    }
}
