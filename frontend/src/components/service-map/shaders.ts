// WebGL Shader definitions for high-performance service map rendering

// Vertex shader for nodes (services)
export const nodeVertexShader = `
  precision mediump float;
  
  attribute vec2 aPosition;
  attribute vec4 aColor;
  attribute float aSize;
  attribute float aGlow;
  
  uniform mat3 uTransform;
  uniform float uPixelRatio;
  uniform float uTime;
  
  varying vec4 vColor;
  varying float vGlow;
  varying vec2 vPosition;
  
  void main() {
    // Apply zoom/pan transform
    vec2 position = (uTransform * vec3(aPosition, 1.0)).xy;
    
    // Add subtle pulsing animation to selected/highlighted nodes
    float sizeMultiplier = 1.0 + (aGlow * 0.1 * sin(uTime * 2.0));
    
    // Set point size with pixel ratio compensation
    gl_PointSize = aSize * sizeMultiplier * uPixelRatio;
    
    // Convert to clip space
    gl_Position = vec4(position, 0.0, 1.0);
    
    // Pass to fragment shader
    vColor = aColor;
    vGlow = aGlow;
    vPosition = aPosition;
  }
`;

// Fragment shader for nodes (services)
export const nodeFragmentShader = `
  precision mediump float;
  
  varying vec4 vColor;
  varying float vGlow;
  
  uniform float uTime;
  
  void main() {
    // Calculate distance from center of point
    vec2 coord = gl_PointCoord - vec2(0.5, 0.5);
    float distance = length(coord);
    
    // Basic circle shape with anti-aliased edges
    float circle = smoothstep(0.5, 0.48, distance);
    
    // Add glow effect based on vGlow intensity
    float glowIntensity = vGlow * 0.6;
    float glow = smoothstep(0.5, 0.35, distance) * glowIntensity;
    
    // Add subtle animation to glow
    float glowPulse = (1.0 + 0.2 * sin(uTime * 3.0 + vGlow * 5.0));
    glow *= glowPulse;
    
    // Create slightly lighter center for 3D effect
    float highlight = smoothstep(0.5, 0.2, distance) * 0.3;
    
    // Compose final color with glow
    vec4 baseColor = vColor;
    vec4 glowColor = vec4(baseColor.rgb * 1.2, baseColor.a * 0.7);
    vec4 finalColor = mix(baseColor, glowColor, glow);
    
    // Add highlight
    finalColor.rgb += highlight;
    
    // Apply circle mask with alpha
    gl_FragColor = finalColor;
    gl_FragColor.a *= circle;
    
    // Discard fully transparent pixels
    if (gl_FragColor.a < 0.01) discard;
  }
`;

// Vertex shader for edges (connections)
export const edgeVertexShader = `
  precision mediump float;
  
  attribute vec2 aPosition;
  attribute vec4 aColor;
  attribute float aWidth;
  
  uniform mat3 uTransform;
  uniform float uPixelRatio;
  
  varying vec4 vColor;
  
  void main() {
    // Apply zoom/pan transform
    vec2 position = (uTransform * vec3(aPosition, 1.0)).xy;
    
    // Set position
    gl_Position = vec4(position, 0.0, 1.0);
    
    // Pass to fragment shader
    vColor = aColor;
  }
`;

// Fragment shader for edges (connections)
export const edgeFragmentShader = `
  precision mediump float;
  
  varying vec4 vColor;
  
  void main() {
    gl_FragColor = vColor;
  }
`;

// Vertex shader for data packets moving along edges
export const packetVertexShader = `
  precision mediump float;
  
  attribute vec2 aPosition;
  attribute vec4 aColor;
  attribute float aSize;
  
  uniform mat3 uTransform;
  uniform float uPixelRatio;
  uniform float uTime;
  
  varying vec4 vColor;
  
  void main() {
    // Apply zoom/pan transform
    vec2 position = (uTransform * vec3(aPosition, 1.0)).xy;
    
    // Add subtle size variation
    float sizeVariation = 1.0 + 0.1 * sin(uTime * 10.0 + aPosition.x + aPosition.y);
    
    // Set point size with pixel ratio compensation
    gl_PointSize = aSize * sizeVariation * uPixelRatio;
    
    // Convert to clip space
    gl_Position = vec4(position, 0.0, 1.0);
    
    // Pass to fragment shader
    vColor = aColor;
  }
`;

// Fragment shader for data packets
export const packetFragmentShader = `
  precision mediump float;
  
  varying vec4 vColor;
  
  uniform float uTime;
  
  void main() {
    // Calculate distance from center of point
    vec2 coord = gl_PointCoord - vec2(0.5, 0.5);
    float distance = length(coord);
    
    // Create diamond shape
    float diamond = 1.0 - smoothstep(0.3, 0.32, abs(coord.x) + abs(coord.y));
    
    // Create glowing trail effect
    float trail = smoothstep(0.32, 0.0, abs(coord.x) + abs(coord.y)) * 0.6;
    
    // Add subtle animation to trail
    float trailPulse = (1.0 + 0.3 * sin(uTime * 5.0));
    trail *= trailPulse;
    
    // Compose final color with trail
    vec4 finalColor = vColor;
    finalColor.rgb += trail * finalColor.rgb;
    
    // Apply diamond mask with alpha
    finalColor.a *= diamond;
    
    // Discard fully transparent pixels
    if (finalColor.a < 0.01) discard;
    
    gl_FragColor = finalColor;
  }
`;

// Utility function to compile shader
export function compileShader(gl: WebGLRenderingContext, source: string, type: number): WebGLShader {
  const shader = gl.createShader(type);
  if (!shader) {
    throw new Error('Could not create shader');
  }
  
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const info = gl.getShaderInfoLog(shader);
    gl.deleteShader(shader);
    throw new Error('Error compiling shader: ' + info);
  }
  
  return shader;
}

// Create shader program with attributes and uniforms
export function createShaderProgram(
  gl: WebGLRenderingContext,
  vertexShader: string,
  fragmentShader: string,
  attributes: string[],
  uniforms: string[]
): {
  program: WebGLProgram;
  attributes: Record<string, number>;
  uniforms: Record<string, WebGLUniformLocation>;
} {
  // Compile shaders
  const vertexShaderObj = compileShader(gl, vertexShader, gl.VERTEX_SHADER);
  const fragmentShaderObj = compileShader(gl, fragmentShader, gl.FRAGMENT_SHADER);
  
  // Create program
  const program = gl.createProgram();
  if (!program) {
    throw new Error('Could not create program');
  }
  
  // Attach shaders
  gl.attachShader(program, vertexShaderObj);
  gl.attachShader(program, fragmentShaderObj);
  gl.linkProgram(program);
  
  // Check link status
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const info = gl.getProgramInfoLog(program);
    gl.deleteProgram(program);
    throw new Error('Error linking program: ' + info);
  }
  
  // Get attribute locations
  const attributeLocations: Record<string, number> = {};
  attributes.forEach(attribute => {
    attributeLocations[attribute] = gl.getAttribLocation(program, attribute);
  });
  
  // Get uniform locations
  const uniformLocations: Record<string, WebGLUniformLocation> = {};
  uniforms.forEach(uniform => {
    const location = gl.getUniformLocation(program, uniform);
    if (location) {
      uniformLocations[uniform] = location;
    }
  });
  
  return {
    program,
    attributes: attributeLocations,
    uniforms: uniformLocations,
  };
}

// Create node shader program
export function createNodeProgram(gl: WebGLRenderingContext) {
  return createShaderProgram(
    gl,
    nodeVertexShader,
    nodeFragmentShader,
    ['aPosition', 'aColor', 'aSize', 'aGlow'],
    ['uTransform', 'uPixelRatio', 'uTime']
  );
}

// Create edge shader program
export function createEdgeProgram(gl: WebGLRenderingContext) {
  return createShaderProgram(
    gl,
    edgeVertexShader,
    edgeFragmentShader,
    ['aPosition', 'aColor', 'aWidth'],
    ['uTransform', 'uPixelRatio']
  );
}

// Create packet shader program
export function createPacketProgram(gl: WebGLRenderingContext) {
  return createShaderProgram(
    gl,
    packetVertexShader,
    packetFragmentShader,
    ['aPosition', 'aColor', 'aSize'],
    ['uTransform', 'uPixelRatio', 'uTime']
  );
}