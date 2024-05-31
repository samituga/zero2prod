import { AlbConfig, CertificateConfig, Config, EcsConfig, RdsConfig, VpcConfig } from './type';

const vpc: VpcConfig = {
  maxAzs: 2,
};

const certificate: CertificateConfig = {
  domainName: 'avada7.com',
  scope: 'dev',
};

const alb: AlbConfig = {
  healthCheck: {
    path: '/health_check',
    intervalSec: 30,
    timeoutSec: 5,
    healthyThresholdCount: 5,
    unhealthyThresholdCount: 2,
    healthyHttpCodes: '200',
  },
};

const rds: RdsConfig = {
  allocatedStorage: 20,
  maxAllocatedStorage: 40,
  instanceType: 't3.micro',
  port: 5432,
};

const ecs: EcsConfig = {
  desiredCount: 1,
  taskDefConfig: {
    imageTag: 'latest',
    memoryLimitMiB: 512,
    cpu: 256,
    containerPort: 8080,
    hostPort: 80,
  },
};

export const devConfig: Config = {
  vpc,
  certificate,
  alb,
  ecs,
};
