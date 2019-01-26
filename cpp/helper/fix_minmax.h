//
// Created by alik on 26.01.19.
//

// SDK defines its own max/min, but it creates conflicts.
#include <algorithm>

#ifdef max
    #undef max
#endif
#define max max
using std::max;

#ifdef min
    #undef min
#endif
#define min min
using std::min;