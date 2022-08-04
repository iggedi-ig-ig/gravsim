#!/bin/bash

glslc -O -o shaders/vert.spv src/shaders/vertex.vert && glslc -O -o shaders/frag.spv src/shaders/fragment.frag || exit 1