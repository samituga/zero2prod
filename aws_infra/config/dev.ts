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
    path: '/',
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
    containerPort: 80, // 8080
  },
};

export const devConfig: Config = {
  vpc,
  certificate,
  alb,
  rds,
  ecs,
};
