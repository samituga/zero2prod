import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import { DeploymentControllerType } from 'aws-cdk-lib/aws-ecs/lib/base/base-service';
import { ApplicationTargetGroup } from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Construct } from 'constructs';

interface EcsStackProps extends cdk.StackProps {
  vpc: ec2.Vpc;
  repository: ecr.Repository;
  blueTargetGroup: ApplicationTargetGroup;
  greenTargetGroup: ApplicationTargetGroup;
}

export class EcsStack extends cdk.Stack {
  public readonly ecsCluster: ecs.Cluster;
  public readonly ecsService: ecs.FargateService;

  constructor(scope: Construct, id: string, props: EcsStackProps) {
    super(scope, id, props);

    const { vpc, repository, blueTargetGroup, greenTargetGroup } = props;

    this.ecsCluster = new ecs.Cluster(this, 'MyEcsCluster', {
      vpc: vpc,
    });

    const taskDefinition = new ecs.FargateTaskDefinition(this, 'MyTaskDef');
    taskDefinition.addContainer('MyContainer', {
      image: ecs.ContainerImage.fromEcrRepository(repository, 'latest'),
      essential: true,
      memoryLimitMiB: 512,
      cpu: 256,
      portMappings: [
        {
          containerPort: 8080,
          hostPort: 8080,
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

    this.ecsService = new ecs.FargateService(this, 'MyFargateService', {
      cluster: this.ecsCluster,
      desiredCount: 2,
      deploymentController: { type: DeploymentControllerType.CODE_DEPLOY },
      taskDefinition,
      assignPublicIp: true,
    });

    // Attach the ECS service to the blue target group initially
    this.ecsService.attachToApplicationTargetGroup(blueTargetGroup);
  }
}
