precision mediump float;

// consider adapting https://github.com/chanzuckerberg/cellxgene/blob/main/client/src/components/scatterplot/drawPointsRegl.js

varying lowp vec3 color;

void main() {
    gl_FragColor = vec4(color, 1);
}