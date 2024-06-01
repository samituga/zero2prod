#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import 'source-map-support/register';
import { getConfig, isScope, stackId } from '../config/helper';
import { Scope } from '../config/type';
import { AlbStack } from '../lib/alb-stack';
import { EcrStack } from '../lib/ecr-stack';
import { EcsStack } from '../lib/ecs-stack';
import { RdsStack } from '../lib/rds-stack';
import { SgStack } from '../lib/sg-stack';
import { VpcStack } from '../lib/vpc-stack';
import { CodePipelineStack } from '../lib/code-pipeline-stack';

const envScope = process.env.CDK_DEPLOY_SCOPE;

if (envScope && !isScope(envScope)) {
  throw new Error(`Scope from environment ${envScope} is not valid`);
}

const scope: Scope = envScope && isScope(envScope) ? envScope : 'Dev';

const env = {
  account: process.env.CDK_DEPLOY_ACCOUNT || process.env.CDK_DEFAULT_ACCOUNT,
  region: process.env.CDK_DEPLOY_REGION || process.env.CDK_DEFAULT_REGION,
};

const config = getConfig(scope);

const app = new cdk.App();

cdk.Tags.of(app).add('Scope', scope);
cdk.Tags.of(app).add('Stack', 'zero2prod');

const ecrStack = new EcrStack(app, stackId(scope, 'EcrStack'), {
  env,
});

const vpcStack = new VpcStack(app, stackId(scope, 'VpcStack'), {
  env,
  config: config.vpc,
});
const vpc = vpcStack.vpc;

const sgStack = new SgStack(app, stackId(scope, 'SgStack'), {
  env,
  ecsConfig: config.ecs,
  rdsConfig: config.rds,
  vpc,
});

const rdsStack = new RdsStack(app, stackId(scope, 'RdsStack'), {
  env,
  config: config.rds,
  vpc,
  sg: sgStack.rds,
});

const albStack = new AlbStack(app, stackId(scope, 'AlbStack'), {
  env,
  config: config.alb,
  ecsConfig: config.ecs,
  vpc,
  sg: sgStack.alb,
});

const ecsStack = new EcsStack(app, stackId(scope, 'EcsStack'), {
  env,
  config: config.ecs,
  repository: ecrStack.repository,
  targetGroupBlue: albStack.targetGroupBlue,
  vpc,
  sg: sgStack.ecs,
  rdsProps: rdsStack.dbProps,
});

const codePipelineStack = new CodePipelineStack(app, stackId(scope, 'CodePipelineStack'), {
  env,
  config: config.codePipeline,
  taskDefConfig: config.ecs.taskDefConfig,
  repository: ecrStack.repository,
  rdsProps: rdsStack.dbProps,
  taskDefinition: ecsStack.taskDefinition,
  ecsService: ecsStack.ecsService,
  albListener: albStack.listener80,
  targetGroupBlue: albStack.targetGroupBlue,
  targetGroupGreen: albStack.targetGroupGreen,
});
