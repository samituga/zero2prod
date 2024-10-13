import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as elb from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Construct } from 'constructs';
import { EcsConfig } from '../config/type';
import { RdsInstanceProps } from './rds-stack';

interface EcsStackProps extends cdk.StackProps {
  config: EcsConfig;
  repository: ecr.Repository;
  targetGroupBlue: elb.ApplicationTargetGroup;
  vpc: ec2.Vpc;
  sg: ec2.SecurityGroup;
  rdsProps: RdsInstanceProps;
}

export class EcsStack extends cdk.Stack {
  public readonly ecsCluster: ecs.Cluster;
  public readonly ecsService: ecs.FargateService;
  public readonly taskDefinition: ecs.FargateTaskDefinition;

  constructor(scope: Construct, id: string, props: EcsStackProps) {
    super(scope, id, props);

    const { config, vpc, sg, repository, targetGroupBlue, rdsProps } = props;

    this.ecsCluster = new ecs.Cluster(this, 'EcsCluster', {
      vpc: vpc,
    });

    const taskDefConfig = config.taskDefConfig;
    const healthCheckConfig = taskDefConfig.healthCheck;

    this.taskDefinition = new ecs.FargateTaskDefinition(this, 'TaskDef');
    this.taskDefinition.addContainer('TaskContainer', {
      image: ecs.ContainerImage.fromEcrRepository(repository, taskDefConfig.imageTag),
      essential: true,
      memoryLimitMiB: taskDefConfig.memoryLimitMiB,
      cpu: taskDefConfig.cpu,
      logging: new ecs.AwsLogDriver({ streamPrefix: 'EventDemo', mode: ecs.AwsLogDriverMode.NON_BLOCKING }),
      healthCheck: {
        command: ['CMD-SHELL', healthCheckConfig.command],
        interval: cdk.Duration.seconds(healthCheckConfig.intervalSec),
        retries: healthCheckConfig.unhealthyThresholdCount,
        timeout: cdk.Duration.seconds(healthCheckConfig.timeoutSec),
        startPeriod: cdk.Duration.seconds(healthCheckConfig.startPeriodSec),
      },
      portMappings: [
        {
          containerPort: taskDefConfig.containerPort,
          protocol: ecs.Protocol.TCP,
        },
      ],
      environment: {
        APP_APPLICATION__BASE_URL: "https://there-is-no-such-domain.com", // TODO get the real url

        APP_DATABASE__USERNAME: rdsProps.credentials.username,
        APP_DATABASE__HOST: rdsProps.address,
        APP_DATABASE__PORT: rdsProps.port.toString(),
        APP_DATABASE__DATABASE_NAME: rdsProps.databaseName,
      },
      secrets: {
        APP_DATABASE__PASSWORD: ecs.Secret.fromSecretsManager(rdsProps.credentials.secret!, 'password'),
      },
    });

    this.ecsService = new ecs.FargateService(this, 'FargateService', {
      cluster: this.ecsCluster,
      desiredCount: config.desiredCount,
      deploymentController: { type: ecs.DeploymentControllerType.CODE_DEPLOY },
      taskDefinition: this.taskDefinition,
      assignPublicIp: true,
      securityGroups: [sg],
    });

    // Attach the ECS service to the blue target group initially
    this.ecsService.attachToApplicationTargetGroup(targetGroupBlue);
  }
}
