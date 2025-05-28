/**
 * Types for System Synthesis component
 */

// Root cause analysis types
export interface RootCauseAnalysis {
  rootCause: string;
  impact: string;
  confidence: number;
  triggeredBy: string;
  details: {
    symptom: string;
    affectedServices: string[];
    evidencePoints: string[];
    timeline: TimelineEvent[];
  };
}

export interface TimelineEvent {
  time: string;
  event: string;
}

// Recommendation types
export interface Recommendation {
  id: string;
  type: 'immediate' | 'short-term' | 'long-term';
  action: string;
  impact: 'High' | 'Medium' | 'Low';
  confidence: number;
  difficulty: 'High' | 'Medium' | 'Low';
  implementation: ConfigImplementation | CodeImplementation | ArchitectureImplementation;
}

export interface ConfigImplementation {
  type: 'config';
  file: string;
  changes: { before: string; after: string }[];
}

export interface CodeImplementation {
  type: 'code';
  file?: string;
  description: string;
}

export interface ArchitectureImplementation {
  type: 'architecture';
  description: string;
}

// Simulation types
export interface SimulationResults {
  originalLatency: number;
  predictedLatency: number;
  improvementPercent: number;
  throughputIncrease: string;
  potentialIssues: string[];
}

// Component props
export interface SystemSynthesisProps {
  rootCauseData?: RootCauseAnalysis | null;
  showActions?: boolean;
}