import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import { DeploymentControllerType } from 'aws-cdk-lib/aws-ecs/lib/base/base-service';
import { ApplicationTargetGroup } from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Construct } from 'constructs';
import { EcsConfig } from '../config/type';

interface EcsStackProps extends cdk.StackProps {
  config: EcsConfig;
  repository: ecr.Repository;
  targetGroupBlue: ApplicationTargetGroup;
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
      image: ecs.ContainerImage.fromEcrRepository(repository, 'latest'),
      essential: true,
      memoryLimitMiB: taskDefConfig.memoryLimitMiB,
      cpu: taskDefConfig.cpu,
      portMappings: [
        {
          containerPort: taskDefConfig.containerPort,
          hostPort: taskDefConfig.hostPort,
          protocol: ecs.Protocol.TCP,
        },
      ],
      environment: {
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
      deploymentController: { type: DeploymentControllerType.CODE_DEPLOY },
      taskDefinition,
      assignPublicIp: true,
      securityGroups: [sg],
    });

    // Attach the ECS service to the blue target group initially
    this.ecsService.attachToApplicationTargetGroup(targetGroupBlue);
  }
}
