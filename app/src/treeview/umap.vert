
precision mediump float;

attribute vec2 a_position;
attribute vec3 a_color;


varying highp vec3 color;
///// attribute vec3 color;


uniform float u_camera_x;
uniform float u_camera_y;

uniform float u_camera_zoom_x;
uniform float u_camera_zoom_y;

uniform float u_display_w;
uniform float u_display_h;


void main() {

    //Transform from world coordinates to [-1,1] camera coordinates
    vec2 a_cam_pos = vec2(u_camera_x, u_camera_y);
    vec2 a_position2 = vec2(a_position.x, a_position.y);
    vec2 a_position3 = a_position2 - a_cam_pos;

    vec2 u_camera_zoom = vec2(u_camera_zoom_x, u_camera_zoom_y);
    vec2 scaled = a_position3 * u_camera_zoom;

    gl_Position = vec4(scaled.x, -scaled.y, 0.0, 1.0);   // Invert camera y to match 

    //Set size of points
    gl_PointSize = 2.0;

    //Set color
//    color = vec3(0.0, 0.0, 0.0);
    color = a_color;

}


