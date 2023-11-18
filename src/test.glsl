#version 450

// Invocations in the (x, y, z) dimension
layout(local_size_x = 8, local_size_y = 1, local_size_z = 1) in;

layout(binding = 0) 
buffer InBuff {
    float data[];
} in_buff;

void main() {
    uint i = gl_GlobalInvocationID.x;

    in_buff.data[i] += 1.5; 
}
