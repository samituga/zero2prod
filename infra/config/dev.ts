import {
  AlbConfig,
  CertificateConfig,
  CodePipelineConfig,
  Config,
  EcsConfig,
  RdsConfig,
  VpcConfig,
} from './type';

const vpc: VpcConfig = {
  maxAzs: 2,
};

const certificate: CertificateConfig = {
  domainName: 'avada7.com',
  scope: 'dev',
};

const healthCheckPath = '/health_check';

const alb: AlbConfig = {
  healthCheck: {
    path: healthCheckPath,
    intervalSec: 90,
    timeoutSec: 3,
    healthyThresholdCount: 4,
    unhealthyThresholdCount: 2,
    healthyHttpCodes: '200',
  },
};

const rds: RdsConfig = {
  allocatedStorage: 20,
  maxAllocatedStorage: 40,
  instanceType: 't3.micro',
  username: 'zero2prod',
  databaseName: 'newsletter',
  port: 5432,
};

const containerPort = 8080;

const ecs: EcsConfig = {
  desiredCount: 1,
  taskDefConfig: {
    imageTag: 'latest',
    memoryLimitMiB: 512,
    cpu: 256,
    containerPort,
    healthCheck: {
      command: `wget --no-verbose --tries=1 http://localhost:${containerPort}${healthCheckPath} || exit 1`,
      intervalSec: 30,
      timeoutSec: 2,
      unhealthyThresholdCount: 4,
      startPeriodSec: 30,
    },
  },
};

const codePipeline: CodePipelineConfig = {
  codeSource: {
    owner: 'samituga',
    repo: 'zero2prod',
    branch: 'aws-infra', // TODO main
  },
};

export const devConfig: Config = {
  vpc,
  certificate,
  alb,
  rds,
  ecs,
  codePipeline,
};
