// Input RNG data
StructuredBuffer<uint> rngValues : register(t0, space0);

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


// Gets the float value from a 32-bit int RNG value
float rng_to_float(uint value, float range) 
{
    return value * 2.3283064365386963e-10 * range;
}

// Returns whether a point is within the given ellipse, given the ellipse bounds (using integer rounding)
bool point_in_ellipse(int left, int top, int right, int bottom, int x, int y)
{
    float dx = ((float)x - (float)((right + left) / 2)) / (float)((right - left) / 2);
    float dy = ((float)y - (float)((bottom + top) / 2)) / (float)((bottom - top) / 2);
    return (dx * dx) + (dy * dy) <= 1.0;
}

// Returns whether a snowball at the given position is colliding with the given bounding box
bool snowball_colliding(float x, float y, int bboxLeft, int bboxTop, int bboxRight, int bboxBottom)
{
    const float radius = 2.0;
    float x1 = x - radius;
    float y1 = y - radius;
    float x2 = x + radius;
    float y2 = y + radius;
    if (y2 < (float)bboxTop)
    {
        return false;
    }
    if (y1 >= (float)(bboxBottom + 1)) 
    {
        return false;
    }
    if (x1 >= (float)(bboxRight + 1)) 
    {
        return false;
    }
    if (x2 < (float)bboxLeft) 
    {
        return false;
    }
    float centerX = (x1 + x2) / 2.0;
    float centerY = (y1 + y2) / 2.0;
    if (((centerX < (float)bboxLeft || (float)bboxRight < centerX)) &&
         (centerY < (float)bboxTop || (float)bboxBottom < centerY)) 
    {
        int ellipseLeft = (int)round(x1);
        int ellipseTop = (int)round(y1);
        int ellipseRight = (int)round(x2);
        int ellipseBottom = (int)round(y2);
        int bboxCheckX = (float)bboxRight <= centerX ? bboxRight : bboxLeft;
        int bboxCheckY = (float)bboxBottom <= centerY ? bboxBottom : bboxTop;
        return point_in_ellipse(ellipseLeft, ellipseTop, ellipseRight, ellipseBottom, bboxCheckX, bboxCheckY);
    }
    return true;
}

#define PROCESS_SNOWBALL \
    uint originalSnowballMoveAmounts = snowballMoveAmounts[j >> 3]; \
    uint snowballMoveAmount = (originalSnowballMoveAmounts >> ((j << 2) & 31)) & 15; \
    if ((snowballMoveAmount & 14) == 14) \
    { \
        continue; \
    } \
    float snowballY = snowballPositions[(j << 1) + 1]; \
    if ((int)snowballY > bboxBottom + 10) \
    { \
        continue; \
    } \
    float snowballX = snowballPositions[j << 1]; \
    if (snowballMoveAmount == 0 && (int)snowballY < bboxTop - 4) \
    { \
        snowballMoveAmount = snowballX < 150.0 ? 14 : 15; \
        snowballMoveAmounts[j >> 3] = (originalSnowballMoveAmounts & ~((uint)15 << ((j << 2) & 31))) | (snowballMoveAmount << ((j << 2) & 31)); \
        continue; \
    } \
    if (snowball_colliding(snowballX, snowballY, bboxLeft, bboxTop, bboxRight, bboxBottom)) \
    { \
        snowballMoveAmount = (uint)floor(rng_to_float(rngValues[rngPosition], 4.0)) + 1; \
        rngPosition++; \
    } \
    if (snowballMoveAmount != 0) \
    { \
        if ((float)bboxLeft > snowballX) \
        { \
            snowballX -= (float)(snowballMoveAmount + 1); \
        } \
        else if ((float)bboxRight < snowballX) \
        { \
            snowballX += (float)(snowballMoveAmount + 1); \
        } \
        if ((float)bboxTop > snowballY) \
        { \
            snowballY -= (float)(snowballMoveAmount + 1); \
        } \
        else if ((float)bboxBottom < snowballY) \
        { \
            snowballY += (float)(snowballMoveAmount + 1); \
        } \
        snowballX += ((rng_to_float(rngValues[rngPosition], (float)(snowballMoveAmount + 1)) - ((float)(snowballMoveAmount + 1) / 2.0)) / 2.0); \
        rngPosition++; \
        snowballY += ((rng_to_float(rngValues[rngPosition], (float)(snowballMoveAmount + 1)) - ((float)(snowballMoveAmount + 1) / 2.0)) / 2.0); \
        rngPosition++; \
        snowballMoveAmount -= 1; \
        snowballPositions[j << 1] = snowballX; \
        snowballPositions[(j << 1) + 1] = snowballY; \
        snowballMoveAmounts[j >> 3] = (originalSnowballMoveAmounts & ~((uint)15 << ((j << 2) & 31))) | (snowballMoveAmount << ((j << 2) & 31)); \
    }

[numthreads(64, 1, 1)]
void main(uint3 GlobalInvocationID : SV_DispatchThreadID)
{
    // Get RNG position
    uint rngPosition = (uint)(GlobalInvocationID.x);
    uint startRngPosition = rngPosition;

    // Initialize simulation data
    float snowballPositions[240] = 
    { 
        142.2, 441.0, 146.2, 441.0, 150.2, 441.0, 154.2, 441.0, 158.0, 441.0, 142.2, 445.0, 146.2, 445.0, 150.2, 445.0, 154.2, 445.0, 158.0, 445.0,
        142.2, 449.0, 146.2, 449.0, 150.2, 449.0, 154.2, 449.0, 158.0, 449.0, 142.2, 453.0, 146.2, 453.0, 150.2, 453.0, 154.2, 453.0, 158.0, 453.0, 
        142.2, 457.0, 146.2, 457.0, 150.2, 457.0, 154.2, 457.0, 158.0, 457.0, 138.0, 441.0, 138.0, 445.0, 138.0, 449.0, 138.0, 453.0, 138.0, 457.0, 
        142.2, 381.0, 146.2, 381.0, 150.2, 381.0, 154.2, 381.0, 158.0, 381.0, 142.2, 385.0, 146.2, 385.0, 150.2, 385.0, 154.2, 385.0, 158.0, 385.0, 
        142.2, 389.0, 146.2, 389.0, 150.2, 389.0, 154.2, 389.0, 158.0, 389.0, 142.2, 393.0, 146.2, 393.0, 150.2, 393.0, 154.2, 393.0, 158.0, 393.0, 
        142.2, 397.0, 146.2, 397.0, 150.2, 397.0, 154.2, 397.0, 158.0, 397.0, 138.0, 381.0, 138.0, 385.0, 138.0, 389.0, 138.0, 393.0, 138.0, 397.0, 
        142.2, 421.0, 146.2, 421.0, 150.2, 421.0, 154.2, 421.0, 158.0, 421.0, 142.2, 425.0, 146.2, 425.0, 150.2, 425.0, 154.2, 425.0, 158.0, 425.0, 
        142.2, 429.0, 146.2, 429.0, 150.2, 429.0, 154.2, 429.0, 158.0, 429.0, 142.2, 433.0, 146.2, 433.0, 150.2, 433.0, 154.2, 433.0, 158.0, 433.0, 
        142.2, 437.0, 146.2, 437.0, 150.2, 437.0, 154.2, 437.0, 158.0, 437.0, 138.0, 421.0, 138.0, 425.0, 138.0, 429.0, 138.0, 433.0, 138.0, 437.0, 
        142.2, 401.0, 146.2, 401.0, 150.2, 401.0, 154.2, 401.0, 158.0, 401.0, 142.2, 405.0, 146.2, 405.0, 150.2, 405.0, 154.2, 405.0, 158.0, 405.0, 
        142.2, 409.0, 146.2, 409.0, 150.2, 409.0, 154.2, 409.0, 158.0, 409.0, 142.2, 413.0, 146.2, 413.0, 150.2, 413.0, 154.2, 413.0, 158.0, 413.0, 
        142.2, 417.0, 146.2, 417.0, 150.2, 417.0, 154.2, 417.0, 158.0, 417.0, 138.0, 401.0, 138.0, 405.0, 138.0, 409.0, 138.0, 413.0, 138.0, 417.0 
    };
    uint snowballMoveAmounts[15] = 
    { 
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    };

    int bboxLeft = 140, bboxTop = 370, bboxRight = 159, bboxBottom = 380;

    // Simulate snowball movements starting at current RNG position
    for (uint i = 0; i < 10; i++)
    {
        // Update all snowballs
        for (uint j = 30; j < 60; j++)
        {
            PROCESS_SNOWBALL
        }
        for (uint j = 90; j < 120; j++)
        {
            PROCESS_SNOWBALL
        }

        // Move bounding box down
        bboxTop += 3;
        bboxBottom += 3;
    }
    for (uint i = 0; i < 30; i++)
    {
        // Update all snowballs
        for (uint j = 0; j < 120; j++)
        {
            PROCESS_SNOWBALL
        }

        // Move bounding box down
        bboxTop += 3;
        bboxBottom += 3;
    }

    // Compare matching positions
    for (uint i = 0; i < matchPositionsCount; i++)
    {
        uint mpos = matchPositions[i];
        float matchX = (float)(mpos >> 16);
        float matchY = (float)(mpos & 0xffff);
        bool foundMatch = false;
        for (uint j = 0; j < 120; j++)
        {
            uint snowballMoveAmount = (snowballMoveAmounts[j >> 3] >> ((j << 2) & 31)) & 15;
            if (snowballMoveAmount == 14)
            {
                continue;
            }
            float snowballX = snowballPositions[j << 1];
            if (snowballX < 150.0)
            {
                continue;
            }
            float snowballY = snowballPositions[(j << 1) + 1];
            float dx = snowballX - matchX;
            float dy = snowballY - matchY;
            float squaredDistance = (dx * dx) + (dy * dy);
            if (squaredDistance <= 4.0)
            {
                foundMatch = true;
                break;
            }
        }
        if (!foundMatch)
        {
            return;
        }
    }

    // If we matched, increment the number of matches, and track the position
    InterlockedAdd(outBuffer[0], 1);
    outBuffer[1] = startRngPosition;
}