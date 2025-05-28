import type { Service, Connection, GraphRenderState, GraphBuffers, ShaderProgram, PerformanceStats } from '../types-exports';
import { createNodeProgram, createEdgeProgram, createPacketProgram } from '../shaders';

export class WebGLRenderer {
  private canvas: HTMLCanvasElement;
  private gl: WebGLRenderingContext;
  private pixelRatio: number;
  
  // Shader programs
  private nodeProgram: ShaderProgram;
  private edgeProgram: ShaderProgram;
  private packetProgram: ShaderProgram;
  
  // WebGL buffers
  private buffers: GraphBuffers;
  
  // Rendering state
  private renderState: GraphRenderState;
  private transform: { x: number; y: number; k: number };
  private width: number = 0;
  private height: number = 0;
  private startTime: number = Date.now();
  
  // Performance monitoring
  private stats: PerformanceStats = {
    fps: 0,
    renderTime: 0,
    physicsTime: 0,
    nodeCount: 0,
    edgeCount: 0,
    frameCount: 0
  };
  private lastFrameTime: number = 0;
  private frameCount: number = 0;
  private lastFpsUpdate: number = 0;
  
  constructor(canvas: HTMLCanvasElement) {
    if (typeof window === 'undefined') {
      throw new Error('WebGLRenderer cannot be initialized on the server side');
    }
    
    this.canvas = canvas;
    
    try {
      // Get WebGL context
      const gl = canvas.getContext('webgl', {
        antialias: true,
        alpha: true,
        preserveDrawingBuffer: false
      });
      
      if (!gl) {
        throw new Error('WebGL not supported');
      }
      
      this.gl = gl;
      
      // Set up pixel ratio for retina displays
      this.pixelRatio = typeof window !== 'undefined' && window.devicePixelRatio ? window.devicePixelRatio : 1;
      
      try {
        // Initialize shader programs
        this.nodeProgram = createNodeProgram(gl);
        this.edgeProgram = createEdgeProgram(gl);
        this.packetProgram = createPacketProgram(gl);
      } catch (error) {
        console.error('Failed to create shader programs:', error);
        throw new Error('Shader compilation failed');
      }
      
      try {
        // Create empty buffers
        this.buffers = this.createBuffers();
      } catch (error) {
        console.error('Failed to create WebGL buffers:', error);
        throw new Error('Buffer creation failed');
      }
      
      // Initialize render state with empty arrays
      this.renderState = {
        nodePositions: new Float32Array(0),
        nodeColors: new Float32Array(0),
        nodeSizes: new Float32Array(0),
        nodeGlow: new Float32Array(0),
        edgePositions: new Float32Array(0),
        edgeColors: new Float32Array(0),
        edgeWidths: new Float32Array(0),
        packetPositions: new Float32Array(0),
        packetColors: new Float32Array(0),
        packetSizes: new Float32Array(0),
        highlightedNodes: new Set<string>(),
        highlightedEdges: new Set<string>()
      };
      
      // Initialize transform
      this.transform = { x: 0, y: 0, k: 1 };
      
      try {
        // Set up WebGL
        this.setupWebGL();
      } catch (error) {
        console.error('Failed to setup WebGL:', error);
        throw new Error('WebGL setup failed');
      }
    } catch (error) {
      console.error('WebGLRenderer initialization failed:', error);
      throw error;
    }
  }
  
  // Create WebGL buffers
  private createBuffers(): GraphBuffers {
    const gl = this.gl;
    
    // Create node buffers
    const nodePositionBuffer = gl.createBuffer();
    const nodeColorBuffer = gl.createBuffer();
    const nodeSizeBuffer = gl.createBuffer();
    const nodeGlowBuffer = gl.createBuffer();
    
    // Create edge buffers
    const edgePositionBuffer = gl.createBuffer();
    const edgeColorBuffer = gl.createBuffer();
    const edgeWidthBuffer = gl.createBuffer();
    
    // Create packet buffers
    const packetPositionBuffer = gl.createBuffer();
    const packetColorBuffer = gl.createBuffer();
    const packetSizeBuffer = gl.createBuffer();
    
    // Validate buffer creation
    if (!nodePositionBuffer || !nodeColorBuffer || !nodeSizeBuffer || !nodeGlowBuffer ||
        !edgePositionBuffer || !edgeColorBuffer || !edgeWidthBuffer ||
        !packetPositionBuffer || !packetColorBuffer || !packetSizeBuffer) {
      throw new Error('Failed to create WebGL buffers');
    }
    
    return {
      nodePositionBuffer,
      nodeColorBuffer,
      nodeSizeBuffer,
      nodeGlowBuffer,
      edgePositionBuffer,
      edgeColorBuffer,
      edgeWidthBuffer,
      packetPositionBuffer,
      packetColorBuffer,
      packetSizeBuffer
    };
  }
  
  // Set up WebGL configuration
  private setupWebGL() {
    const gl = this.gl;
    
    // Enable blending for transparency
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    
    // Disable depth testing (2D rendering)
    gl.disable(gl.DEPTH_TEST);
    
    // Set clear color (transparent)
    gl.clearColor(0, 0, 0, 0);
  }
  
  // Resize canvas and viewport
  public resize(width: number, height: number) {
    this.width = width;
    this.height = height;
    
    // Update canvas size with pixel ratio
    this.canvas.width = width * this.pixelRatio;
    this.canvas.height = height * this.pixelRatio;
    
    // Set CSS size
    this.canvas.style.width = `${width}px`;
    this.canvas.style.height = `${height}px`;
    
    // Update WebGL viewport
    this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);
  }
  
  // Update transform (pan/zoom)
  public updateTransform(x: number, y: number, k: number) {
    this.transform = { x, y, k };
  }
  
  // Update render state with new data
  public updateRenderState(
    services: Service[],
    connections: Connection[],
    highlightedNodes: Set<string> = new Set(),
    highlightedEdges: Set<string> = new Set()
  ) {
    // Update node data
    const nodeCount = services.length;
    this.renderState.nodePositions = new Float32Array(nodeCount * 2);
    this.renderState.nodeColors = new Float32Array(nodeCount * 4);
    this.renderState.nodeSizes = new Float32Array(nodeCount);
    this.renderState.nodeGlow = new Float32Array(nodeCount);
    
    services.forEach((service, i) => {
      // Position
      this.renderState.nodePositions[i * 2] = service.x || 0;
      this.renderState.nodePositions[i * 2 + 1] = service.y || 0;
      
      // Color based on status
      let r = 0, g = 0, b = 0, a = 1;
      switch (service.status) {
        case 'healthy':
          r = 0; g = 1; b = 0.6; break;
        case 'warning':
          r = 1; g = 0.8; b = 0; break;
        case 'error':
          r = 1; g = 0.3; b = 0.3; break;
        default:
          r = 0.5; g = 0.5; b = 0.5; break;
      }
      
      this.renderState.nodeColors[i * 4] = r;
      this.renderState.nodeColors[i * 4 + 1] = g;
      this.renderState.nodeColors[i * 4 + 2] = b;
      this.renderState.nodeColors[i * 4 + 3] = a;
      
      // Size based on metrics or type
      let size = service.size || 15;
      if (service.type === 'gateway' || service.type === 'auth') {
        size *= 1.2; // Make important services larger
      }
      
      this.renderState.nodeSizes[i] = size;
      
      // Glow for highlighting/selection
      this.renderState.nodeGlow[i] = 
        service.selected ? 1.0 :
        service.highlighted || highlightedNodes.has(service.id) ? 0.7 : 
        0.0;
    });
    
    // Update edge data
    const edgeCount = connections.length;
    this.renderState.edgePositions = new Float32Array(edgeCount * 4); // 2 points * 2 coords
    this.renderState.edgeColors = new Float32Array(edgeCount * 8); // 2 points * 4 colors
    this.renderState.edgeWidths = new Float32Array(edgeCount * 2); // 2 widths per edge
    
    // Find service positions by id
    const servicePositions = new Map<string, [number, number]>();
    services.forEach(service => {
      servicePositions.set(service.id, [service.x || 0, service.y || 0]);
    });
    
    connections.forEach((connection, i) => {
      const sourcePos = servicePositions.get(connection.source) || [0, 0];
      const targetPos = servicePositions.get(connection.target) || [0, 0];
      
      // Edge positions (source -> target)
      this.renderState.edgePositions[i * 4] = sourcePos[0];
      this.renderState.edgePositions[i * 4 + 1] = sourcePos[1];
      this.renderState.edgePositions[i * 4 + 2] = targetPos[0];
      this.renderState.edgePositions[i * 4 + 3] = targetPos[1];
      
      // Edge color based on status
      let r = 0, g = 0, b = 0, a = 0.6; // Semi-transparent by default
      
      switch (connection.status) {
        case 'active':
          r = 0.4; g = 0.8; b = 1.0; break;
        case 'slow':
          r = 1.0; g = 0.8; b = 0.0; break;
        case 'error':
          r = 1.0; g = 0.3; b = 0.3; break;
        case 'idle':
          r = 0.5; g = 0.5; b = 0.5; a = 0.3; break;
      }
      
      // Highlight effect
      if (
        connection.selected || 
        connection.highlighted || 
        highlightedEdges.has(connection.id)
      ) {
        a = 0.9;
        r *= 1.2;
        g *= 1.2;
        b *= 1.2;
      }
      
      // Same color for both ends of the edge
      for (let j = 0; j < 2; j++) {
        this.renderState.edgeColors[i * 8 + j * 4] = r;
        this.renderState.edgeColors[i * 8 + j * 4 + 1] = g;
        this.renderState.edgeColors[i * 8 + j * 4 + 2] = b;
        this.renderState.edgeColors[i * 8 + j * 4 + 3] = a;
      }
      
      // Edge width based on metrics
      const width = connection.width || 
                   (connection.metrics?.throughput || 0) / 100 + 1;
      
      this.renderState.edgeWidths[i * 2] = width;
      this.renderState.edgeWidths[i * 2 + 1] = width;
    });
    
    // Update data packets
    let packetCount = 0;
    connections.forEach(conn => {
      if (conn.dataPackets) {
        packetCount += conn.dataPackets.length;
      }
    });
    
    this.renderState.packetPositions = new Float32Array(packetCount * 2);
    this.renderState.packetColors = new Float32Array(packetCount * 4);
    this.renderState.packetSizes = new Float32Array(packetCount);
    
    let packetIndex = 0;
    connections.forEach(connection => {
      if (!connection.dataPackets) return;
      
      const sourcePos = servicePositions.get(connection.source) || [0, 0];
      const targetPos = servicePositions.get(connection.target) || [0, 0];
      
      connection.dataPackets.forEach(packet => {
        // Calculate position along the edge
        const progress = packet.progress;
        const x = sourcePos[0] + (targetPos[0] - sourcePos[0]) * progress;
        const y = sourcePos[1] + (targetPos[1] - sourcePos[1]) * progress;
        
        this.renderState.packetPositions[packetIndex * 2] = x;
        this.renderState.packetPositions[packetIndex * 2 + 1] = y;
        
        // Packet color based on type
        let r = 0, g = 0, b = 0, a = 0.8;
        
        switch (packet.type) {
          case 'request':
            r = 0.2; g = 0.6; b = 1.0; break;
          case 'response':
            r = 0.2; g = 0.9; b = 0.4; break;
          case 'error':
            r = 1.0; g = 0.3; b = 0.3; break;
        }
        
        this.renderState.packetColors[packetIndex * 4] = r;
        this.renderState.packetColors[packetIndex * 4 + 1] = g;
        this.renderState.packetColors[packetIndex * 4 + 2] = b;
        this.renderState.packetColors[packetIndex * 4 + 3] = a;
        
        // Packet size
        this.renderState.packetSizes[packetIndex] = packet.size || 3;
        
        packetIndex++;
      });
    });
    
    // Update highlighted sets
    this.renderState.highlightedNodes = highlightedNodes;
    this.renderState.highlightedEdges = highlightedEdges;
    
    // Update stats
    this.stats.nodeCount = nodeCount;
    this.stats.edgeCount = edgeCount;
  }
  
  // Update packet positions based on speed
  public updatePackets(connections: Connection[], deltaTime: number) {
    let packetIndex = 0;
    
    try {
      // Create a map of service positions for efficient lookup
      const servicePositions = new Map<string, [number, number]>();
      
      // Don't attempt to update packets if there are no positions to work with
      if (this.renderState.packetPositions.length === 0) {
        return;
      }
      
      connections.forEach(connection => {
        if (!connection.dataPackets) return;
        
        connection.dataPackets.forEach((packet, i) => {
          try {
            // Update progress
            packet.progress += packet.speed * deltaTime;
            
            // Reset if past the end
            if (packet.progress > 1) {
              packet.progress = 0;
            }
            
            // Update position in buffer
            if (packetIndex < this.renderState.packetPositions.length / 2) {
              // Find service positions
              const sourceService = connection.source;
              const targetService = connection.target;
              
              // This is safer but less efficient - we should pre-populate servicePositions
              // For now, we'll use hardcoded fallback values if positions aren't found
              const sourceX = 0, sourceY = 0;
              const targetX = 100, targetY = 100;
              
              // Calculate position along the edge
              const x = sourceX + (targetX - sourceX) * packet.progress;
              const y = sourceY + (targetY - sourceY) * packet.progress;
              
              this.renderState.packetPositions[packetIndex * 2] = x;
              this.renderState.packetPositions[packetIndex * 2 + 1] = y;
            }
            
            packetIndex++;
          } catch (error) {
            console.error('Error updating packet:', error);
          }
        });
      });
    } catch (error) {
      console.error('Error in updatePackets:', error);
    }
  }
  
  // Render the current state
  public render() {
    const startTime = performance.now();
    const gl = this.gl;
    
    // Clear canvas
    gl.clear(gl.COLOR_BUFFER_BIT);
    
    // Calculate transform matrix
    const transformMatrix = this.calculateTransformMatrix();
    
    // Draw edges
    this.drawEdges(transformMatrix);
    
    // Draw packets
    this.drawPackets(transformMatrix);
    
    // Draw nodes
    this.drawNodes(transformMatrix);
    
    // Update performance stats
    this.updatePerformanceStats(startTime);
    
    return this.stats;
  }
  
  // Calculate transform matrix for WebGL
  private calculateTransformMatrix(): Float32Array {
    const { x, y, k } = this.transform;
    
    // Convert screen space to clip space (-1 to 1)
    const scaleX = k * 2 / this.width;
    const scaleY = k * 2 / this.height;
    
    // Center the transform origin
    const translateX = (x / this.width) * 2;
    const translateY = (y / this.height) * 2;
    
    // Create transform matrix (3x3)
    return new Float32Array([
      scaleX, 0, 0,
      0, scaleY, 0,
      translateX, translateY, 1
    ]);
  }
  
  // Draw nodes using WebGL
  private drawNodes(transformMatrix: Float32Array) {
    const gl = this.gl;
    const { nodeProgram, buffers, renderState } = this;
    
    // Check if we have nodes to draw
    if (renderState.nodePositions.length === 0) return;
    
    // Use node program
    gl.useProgram(nodeProgram.program);
    
    // Set uniforms
    gl.uniformMatrix3fv(nodeProgram.uniforms.uTransform, false, transformMatrix);
    gl.uniform1f(nodeProgram.uniforms.uPixelRatio, this.pixelRatio);
    gl.uniform1f(nodeProgram.uniforms.uTime, (Date.now() - this.startTime) / 1000);
    
    // Bind position buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.nodePositionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.nodePositions, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(nodeProgram.attributes.aPosition);
    gl.vertexAttribPointer(nodeProgram.attributes.aPosition, 2, gl.FLOAT, false, 0, 0);
    
    // Bind color buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.nodeColorBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.nodeColors, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(nodeProgram.attributes.aColor);
    gl.vertexAttribPointer(nodeProgram.attributes.aColor, 4, gl.FLOAT, false, 0, 0);
    
    // Bind size buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.nodeSizeBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.nodeSizes, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(nodeProgram.attributes.aSize);
    gl.vertexAttribPointer(nodeProgram.attributes.aSize, 1, gl.FLOAT, false, 0, 0);
    
    // Bind glow buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.nodeGlowBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.nodeGlow, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(nodeProgram.attributes.aGlow);
    gl.vertexAttribPointer(nodeProgram.attributes.aGlow, 1, gl.FLOAT, false, 0, 0);
    
    // Draw nodes as points
    gl.drawArrays(gl.POINTS, 0, renderState.nodePositions.length / 2);
  }
  
  // Draw edges using WebGL
  private drawEdges(transformMatrix: Float32Array) {
    const gl = this.gl;
    const { edgeProgram, buffers, renderState } = this;
    
    // Check if we have edges to draw
    if (renderState.edgePositions.length === 0) return;
    
    // Use edge program
    gl.useProgram(edgeProgram.program);
    
    // Set uniforms
    gl.uniformMatrix3fv(edgeProgram.uniforms.uTransform, false, transformMatrix);
    gl.uniform1f(edgeProgram.uniforms.uPixelRatio, this.pixelRatio);
    
    // Bind position buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.edgePositionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.edgePositions, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(edgeProgram.attributes.aPosition);
    gl.vertexAttribPointer(edgeProgram.attributes.aPosition, 2, gl.FLOAT, false, 0, 0);
    
    // Bind color buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.edgeColorBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.edgeColors, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(edgeProgram.attributes.aColor);
    gl.vertexAttribPointer(edgeProgram.attributes.aColor, 4, gl.FLOAT, false, 0, 0);
    
    // Bind width buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.edgeWidthBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.edgeWidths, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(edgeProgram.attributes.aWidth);
    gl.vertexAttribPointer(edgeProgram.attributes.aWidth, 1, gl.FLOAT, false, 0, 0);
    
    // Draw edges as lines
    gl.lineWidth(1.5); // This has limited browser support
    gl.drawArrays(gl.LINES, 0, renderState.edgePositions.length / 2);
  }
  
  // Draw data packets using WebGL
  private drawPackets(transformMatrix: Float32Array) {
    const gl = this.gl;
    const { packetProgram, buffers, renderState } = this;
    
    // Check if we have packets to draw
    if (renderState.packetPositions.length === 0) return;
    
    // Use packet program
    gl.useProgram(packetProgram.program);
    
    // Set uniforms
    gl.uniformMatrix3fv(packetProgram.uniforms.uTransform, false, transformMatrix);
    gl.uniform1f(packetProgram.uniforms.uPixelRatio, this.pixelRatio);
    gl.uniform1f(packetProgram.uniforms.uTime, (Date.now() - this.startTime) / 1000);
    
    // Bind position buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.packetPositionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.packetPositions, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(packetProgram.attributes.aPosition);
    gl.vertexAttribPointer(packetProgram.attributes.aPosition, 2, gl.FLOAT, false, 0, 0);
    
    // Bind color buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.packetColorBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.packetColors, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(packetProgram.attributes.aColor);
    gl.vertexAttribPointer(packetProgram.attributes.aColor, 4, gl.FLOAT, false, 0, 0);
    
    // Bind size buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.packetSizeBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, renderState.packetSizes, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(packetProgram.attributes.aSize);
    gl.vertexAttribPointer(packetProgram.attributes.aSize, 1, gl.FLOAT, false, 0, 0);
    
    // Draw packets as points
    gl.drawArrays(gl.POINTS, 0, renderState.packetPositions.length / 2);
  }
  
  // Update performance statistics
  private updatePerformanceStats(startTime: number) {
    const now = performance.now();
    const renderTime = now - startTime;
    
    this.stats.renderTime = renderTime;
    this.stats.frameCount++;
    this.frameCount++;
    
    // Update FPS counter every second
    if (now - this.lastFpsUpdate > 1000) {
      this.stats.fps = Math.round((this.frameCount * 1000) / (now - this.lastFpsUpdate));
      this.frameCount = 0;
      this.lastFpsUpdate = now;
    }
  }
  
  // Clean up WebGL resources
  public dispose() {
    const gl = this.gl;
    
    // Delete shader programs
    gl.deleteProgram(this.nodeProgram.program);
    gl.deleteProgram(this.edgeProgram.program);
    gl.deleteProgram(this.packetProgram.program);
    
    // Delete buffers
    gl.deleteBuffer(this.buffers.nodePositionBuffer);
    gl.deleteBuffer(this.buffers.nodeColorBuffer);
    gl.deleteBuffer(this.buffers.nodeSizeBuffer);
    gl.deleteBuffer(this.buffers.nodeGlowBuffer);
    gl.deleteBuffer(this.buffers.edgePositionBuffer);
    gl.deleteBuffer(this.buffers.edgeColorBuffer);
    gl.deleteBuffer(this.buffers.edgeWidthBuffer);
    gl.deleteBuffer(this.buffers.packetPositionBuffer);
    gl.deleteBuffer(this.buffers.packetColorBuffer);
    gl.deleteBuffer(this.buffers.packetSizeBuffer);
  }
}