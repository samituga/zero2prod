import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as elb from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Construct } from 'constructs';
import { EcsConfig } from '../config/type';

interface EcsStackProps extends cdk.StackProps {
  config: EcsConfig;
  repository: ecr.Repository;
  targetGroupBlue: elb.ApplicationTargetGroup;
  vpc: ec2.Vpc;
  sg: ec2.SecurityGroup;
}

export class EcsStack extends cdk.Stack {
  public readonly ecsCluster: ecs.Cluster;
  public readonly ecsService: ecs.FargateService;

  constructor(scope: Construct, id: string, props: EcsStackProps) {
    super(scope, id, props);

    const { config, vpc, sg, repository, targetGroupBlue } = props;

    this.ecsCluster = new ecs.Cluster(this, 'EcsCluster', {
      vpc: vpc,
    });

    const taskDefConfig = config.taskDefConfig;

    const taskDefinition = new ecs.FargateTaskDefinition(this, 'TaskDef');
    taskDefinition.addContainer('TaskContainer', {
      image: ecs.ContainerImage.fromRegistry('yeasy/simple-web'), //  ecs.ContainerImage.fromEcrRepository(repository, 'latest')
      essential: true,
      memoryLimitMiB: taskDefConfig.memoryLimitMiB,
      cpu: taskDefConfig.cpu,
      logging: new ecs.AwsLogDriver({ streamPrefix: 'EventDemo', mode: ecs.AwsLogDriverMode.NON_BLOCKING }),
      healthCheck: {
        command: ['CMD-SHELL', `curl -f http://localhost:${taskDefConfig.containerPort}/ || exit 1`],
        interval: cdk.Duration.seconds(30),
        retries: 3,
        timeout: cdk.Duration.seconds(10),
      },
      portMappings: [
        {
          containerPort: taskDefConfig.containerPort,
          protocol: ecs.Protocol.TCP,
        },
      ],
      environment: {
        PORT: taskDefConfig.containerPort.toString(),
        APP_DATABASE__USERNAME: '',
        APP_DATABASE__PASSWORD: '',
        APP_DATABASE__HOST: '',
        APP_DATABASE__PORT: '',
        APP_DATABASE__DATABASE_NAME: '',
      },
    });

    this.ecsService = new ecs.FargateService(this, 'FargateService', {
      cluster: this.ecsCluster,
      desiredCount: config.desiredCount,
      deploymentController: { type: ecs.DeploymentControllerType.CODE_DEPLOY },
      taskDefinition,
      assignPublicIp: true,
      securityGroups: [sg],
    });

    // Attach the ECS service to the blue target group initially
    this.ecsService.attachToApplicationTargetGroup(targetGroupBlue);
  }
}
