export interface Config {
  vpc: VpcConfig;
  certificate: CertificateConfig;
  alb: AlbConfig;
  ecs: EcsConfig;
}

export interface VpcConfig {
  maxAzs: number;
}

export interface CertificateConfig {
  domainName: string;
  scope?: string;
}

export interface AlbConfig {
  healthCheck: HealthCheckConfig;
}

export interface HealthCheckConfig {
  path: string;
  intervalSec: number;
  timeoutSec: number;
  healthyThresholdCount: number;
  unhealthyThresholdCount: number;
  healthyHttpCodes: string;
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
  hostPort: number;
}
