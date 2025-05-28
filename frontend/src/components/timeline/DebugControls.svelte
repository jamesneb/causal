<script lang="ts">
  import { onMount } from 'svelte';
  
  // Props
  export let currentTimestamp: number = Date.now();
  export let startTime: number = Date.now() - 1000;
  export let endTime: number = Date.now();
  export let isPaused: boolean = true;
  export let breakpoints: number[] = [];
  export let onResume = () => {};
  export let onPause = () => {};
  export let onStepOver = () => {};
  export let onStepInto = () => {};
  export let onStepOut = () => {};
  export let onSeek = (time: number) => {};
  
  // Internal state
  let timelineProgress = 0;
  let timelineProgressPercentage = '0%';
  let currentTimeDisplay = '';
  let timelineElement: HTMLElement;
  let hoveringTimeline = false;
  let hoverTimeDisplay = '';
  let hoverTimePosition = 0;
  
  // Watch for current timestamp changes
  $: {
    updateTimelineProgress();
    currentTimeDisplay = formatTimeOffset(currentTimestamp - startTime);
  }
  
  // Update the timeline progress based on current timestamp
  function updateTimelineProgress() {
    const totalDuration = endTime - startTime;
    timelineProgress = Math.min(1, Math.max(0, (currentTimestamp - startTime) / totalDuration));
    timelineProgressPercentage = `${timelineProgress * 100}%`;
  }
  
  // Handle timeline click to seek
  function handleTimelineClick(event: MouseEvent) {
    if (!timelineElement) return;
    
    const rect = timelineElement.getBoundingClientRect();
    const clickPosition = (event.clientX - rect.left) / rect.width;
    const seekTime = startTime + clickPosition * (endTime - startTime);
    
    onSeek(seekTime);
  }
  
  // Handle timeline hover to show time preview
  function handleTimelineHover(event: MouseEvent) {
    if (!timelineElement) return;
    
    hoveringTimeline = true;
    
    const rect = timelineElement.getBoundingClientRect();
    const hoverPosition = (event.clientX - rect.left) / rect.width;
    const hoverTime = startTime + hoverPosition * (endTime - startTime);
    
    hoverTimeDisplay = formatTimeOffset(hoverTime - startTime);
    hoverTimePosition = event.clientX - rect.left;
  }
  
  // Handle mouse leave
  function handleTimelineLeave() {
    hoveringTimeline = false;
  }
  
  // Format time for display as MM:SS.mmm
  function formatTimeOffset(timeMs: number): string {
    if (isNaN(timeMs)) return '00:00.000';
    
    const totalSeconds = Math.floor(timeMs / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    const milliseconds = Math.floor(timeMs % 1000);
    
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}.${milliseconds.toString().padStart(3, '0')}`;
  }
</script>

<div class="debug-controls">
  <div class="timeline-section">
    <div class="time-display">
      <div class="current-time">
        {currentTimeDisplay}
      </div>
      
      {#if hoveringTimeline}
        <div 
          class="hover-time" 
          style="left: {hoverTimePosition}px"
        >
          {hoverTimeDisplay}
        </div>
      {/if}
    </div>
    
    <div 
      class="timeline" 
      bind:this={timelineElement}
      on:click={handleTimelineClick}
      on:mousemove={handleTimelineHover}
      on:mouseleave={handleTimelineLeave}
    >
      <div class="timeline-track">
        <div 
          class="timeline-progress" 
          style="width: {timelineProgressPercentage}"
        ></div>
        
        <!-- Render breakpoints on timeline -->
        {#each breakpoints as breakpointTime}
          {@const breakpointPosition = (breakpointTime - startTime) / (endTime - startTime) * 100}
          {#if breakpointPosition >= 0 && breakpointPosition <= 100}
            <div 
              class="timeline-breakpoint" 
              style="left: {breakpointPosition}%"
              title="Breakpoint at {formatTimeOffset(breakpointTime - startTime)}"
            ></div>
          {/if}
        {/each}
      </div>
    </div>
    
    <div class="time-range">
      <div class="start-time">
        {formatTimeOffset(0)}
      </div>
      <div class="end-time">
        {formatTimeOffset(endTime - startTime)}
      </div>
    </div>
  </div>
  
  <div class="controls-section">
    <button 
      class="control-button"
      title={isPaused ? "Resume Execution" : "Pause Execution"}
      on:click={isPaused ? onResume : onPause}
    >
      {#if isPaused}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <polygon points="5 3 19 12 5 21 5 3"></polygon>
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
          <rect x="6" y="4" width="4" height="16"></rect>
          <rect x="14" y="4" width="4" height="16"></rect>
        </svg>
      {/if}
    </button>
    
    <button 
      class="control-button"
      title="Step Over"
      on:click={onStepOver}
      disabled={!isPaused}
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M5 8h14"></path>
        <path d="M12 3v10"></path>
        <polyline points="15 11 12 13 9 11"></polyline>
        <path d="M19 21H5"></path>
      </svg>
    </button>
    
    <button 
      class="control-button"
      title="Step Into"
      on:click={onStepInto}
      disabled={!isPaused}
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 3v16"></path>
        <polyline points="15 16 12 19 9 16"></polyline>
        <path d="M19 21H5"></path>
      </svg>
    </button>
    
    <button 
      class="control-button"
      title="Step Out"
      on:click={onStepOut}
      disabled={!isPaused}
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 21V5"></path>
        <polyline points="9 8 12 5 15 8"></polyline>
        <path d="M19 21H5"></path>
      </svg>
    </button>
    
    <div class="controls-right">
      <div class="control-label">
        {#if isPaused}
          Execution Paused
        {:else}
          Replaying...
        {/if}
      </div>
      
      <div class="speed-control">
        <label>Speed:</label>
        <select>
          <option value="0.25">0.25x</option>
          <option value="0.5">0.5x</option>
          <option value="1" selected>1x</option>
          <option value="2">2x</option>
          <option value="4">4x</option>
        </select>
      </div>
    </div>
  </div>
</div>

<style>
  .debug-controls {
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border-light);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .timeline-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .time-display {
    display: flex;
    justify-content: center;
    position: relative;
    height: 24px;
  }
  
  .current-time {
    font-family: monospace;
    font-size: 1rem;
    color: var(--electric-teal);
  }
  
  .hover-time {
    position: absolute;
    font-family: monospace;
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.8);
    background-color: var(--bg-tertiary);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    transform: translateX(-50%);
    bottom: 24px;
    pointer-events: none;
  }
  
  .timeline {
    height: 24px;
    position: relative;
    cursor: pointer;
    padding: 0.5rem 0;
  }
  
  .timeline-track {
    width: 100%;
    height: 6px;
    background-color: var(--bg-tertiary);
    border-radius: 3px;
    position: relative;
    overflow: hidden;
  }
  
  .timeline-progress {
    height: 100%;
    background: linear-gradient(90deg, var(--electric-teal), var(--quantum-purple));
    border-radius: 3px;
    transition: width 0.2s;
  }
  
  .timeline-breakpoint {
    position: absolute;
    width: 8px;
    height: 8px;
    background-color: var(--plasma-pink);
    border-radius: 50%;
    top: -1px;
    transform: translateX(-50%);
    cursor: pointer;
  }
  
  .time-range {
    display: flex;
    justify-content: space-between;
    font-family: monospace;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .controls-section {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .control-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 4px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-light);
    color: var(--text-light);
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .control-button:hover:not(:disabled) {
    background-color: var(--bg-primary);
    border-color: var(--electric-teal);
  }
  
  .control-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .controls-right {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  
  .control-label {
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.8);
  }
  
  .speed-control {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
  }
  
  .speed-control label {
    color: rgba(255, 255, 255, 0.7);
  }
  
  .speed-control select {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-light);
    color: var(--text-light);
    border-radius: 4px;
    padding: 0.25rem 0.5rem;
    font-size: 0.85rem;
  }
  
  .speed-control select:focus {
    outline: none;
    border-color: var(--electric-teal);
  }
</style>