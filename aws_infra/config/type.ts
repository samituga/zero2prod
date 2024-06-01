export type Scope = 'Prod' | 'Dev';

export interface Config {
  vpc: VpcConfig;
  certificate: CertificateConfig;
  alb: AlbConfig;
  rds: RdsConfig;
  ecs: EcsConfig;
  codePipeline: CodePipelineConfig;
}

export interface VpcConfig {
  maxAzs: number;
}

export interface CertificateConfig {
  domainName: string;
  scope?: string;
}

export interface AlbConfig {
  healthCheck: AlbHealthCheckConfig;
}

export interface RdsConfig {
  allocatedStorage: number;
  maxAllocatedStorage: number;
  instanceType: string;
  username: string;
  databaseName: string;
  port: number;
}

export interface EcsConfig {
  desiredCount: number;
  taskDefConfig: TaskDefConfig;
}

export interface TaskDefConfig {
  imageTag: string;
  memoryLimitMiB: number;
  cpu: number;
  containerPort: number;
  healthCheck: TaskDefHealthCheckConfig;
}

export interface BaseHealthCheckConfig {
  intervalSec: number;
  timeoutSec: number;
  unhealthyThresholdCount: number;
}

export interface TaskDefHealthCheckConfig extends BaseHealthCheckConfig {
  command: string ;
  startPeriodSec: number;
}

export interface AlbHealthCheckConfig extends BaseHealthCheckConfig {
  path: string;
  healthyThresholdCount: number;
  healthyHttpCodes: string;
}

export interface CodePipelineConfig {
  codeSource: CodeSourceConfig;
}

export interface CodeSourceConfig {
  owner: string;
  repo: string;
  branch: string;
}
