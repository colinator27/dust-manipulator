// Input snowball data
StructuredBuffer<uint> snowballData : register(t0, space0);

// Input matching snowball positions, packed 16 bits X/Y
StructuredBuffer<uint> matchPositions : register(t1, space0);

// Output buffer (just has a few variables written to by any successful matches)
RWStructuredBuffer<uint> outBuffer : register(u0, space1);

// Uniforms
cbuffer uniformBuffer : register(b0, space2)
{
    // Number of matching snowball positions (size of matchPositions)
    uint matchPositionsCount : packoffset(c0);
};

[numthreads(64, 1, 1)]
void main(uint3 GlobalInvocationID : SV_DispatchThreadID)
{
    // Get snowball data position
    uint startRngPosition = (uint)(GlobalInvocationID.x);
    uint snowballDataPosition = startRngPosition << 5;

    // Compare matching positions
    for (uint i = 0; i < matchPositionsCount; i++)
    {
        // 1 matching snowball is packed into a 32-bit int
        uint mpos = matchPositions[i];
        int matchX = (int)(mpos >> 16);
        int matchY = (int)(mpos & 0xffff);
        bool foundMatch = false;
        for (uint j = 0; j < 32; j++)
        {
            // 2 snowballs are packed into a 32-bit int
            uint snowballPairData = snowballData[snowballDataPosition + j];
            int snowballX = (int)(snowballPairData & 0xff);
            int snowballY = (int)((snowballPairData >> 8) & 0xff);
            int dx = snowballX - matchX;
            int dy = snowballY - matchY;
            int squaredDistance = (dx * dx) + (dy * dy);
            snowballX = (int)((snowballPairData >> 16) & 0xff);
            snowballY = (int)(snowballPairData >> 24);
            dx = snowballX - matchX;
            dy = snowballY - matchY;
            if (squaredDistance <= 4)
            {
                foundMatch = true;
                break;
            }
            squaredDistance = (dx * dx) + (dy * dy);
            if (squaredDistance <= 4)
            {
                foundMatch = true;
                break;
            }
        }

        // If no match was found, no longer consider
        if (!foundMatch)
        {
            return;
        }
    }

    // If we matched, increment the number of matches, and track the position
    InterlockedAdd(outBuffer[0], 1);
    outBuffer[1] = startRngPosition;
}