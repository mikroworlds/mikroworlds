import * as twgl from "twgl.js";
import vert from "./glsl/vertex";
import frag from "./glsl/fragment";
import { setWorld } from "./graphics/world";
import { World } from "./world";

twgl.setDefaults({ attribPrefix: "a_" });

const canvas = document.getElementById("game")! as HTMLCanvasElement;
const gl = canvas.getContext("webgl");
if (!gl) throw new Error("Your browser does not support WebGL.");

const programInfo = twgl.createProgramInfo(gl, [vert, frag]);

export const render = (world: World) => {
  const _render = (dt: number) => {
    twgl.resizeCanvasToDisplaySize(gl.canvas);

    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

    const uniforms = {
      u_dt: dt * 0.001,
      u_resolution: [gl.canvas.width, gl.canvas.height],
      u_matrix: twgl.m4.ortho(0, gl.canvas.clientWidth, 0, gl.canvas.clientHeight, -1, 1),
    };

    if (world.background) gl.clearColor(...world.background.toGL());
    else gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.useProgram(programInfo.program);

    const arrays = {
      position: {
        numComponents: 2,
        data: [],
      },
      color: { numComponents: 4, data: [] },
    };
    setWorld(gl, arrays, world);
    const bufferInfo = twgl.createBufferInfoFromArrays(gl, arrays);

    twgl.setBuffersAndAttributes(gl, programInfo, bufferInfo);
    twgl.setUniforms(programInfo, uniforms);

    twgl.drawBufferInfo(gl, bufferInfo);

    requestAnimationFrame(_render);
  };

  requestAnimationFrame(_render);

  return _render;
};
