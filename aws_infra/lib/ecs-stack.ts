import * as cdk from 'aws-cdk-lib';
import {Construct} from 'constructs';
import {Cluster, ContainerImage, FargateService, FargateTaskDefinition} from 'aws-cdk-lib/aws-ecs';
import {ApplicationTargetGroup} from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import {Repository} from 'aws-cdk-lib/aws-ecr';
import {Vpc} from 'aws-cdk-lib/aws-ec2';

interface EcsStackProps extends cdk.StackProps {
    vpc: Vpc;
    repository: Repository;
    blueTargetGroup: ApplicationTargetGroup;
    greenTargetGroup: ApplicationTargetGroup;
}

export class EcsStack extends cdk.Stack {
    public readonly ecsCluster: Cluster;
    public readonly ecsService: FargateService;

    constructor(scope: Construct, id: string, props: EcsStackProps) {
        super(scope, id, props);

        this.ecsCluster = new Cluster(this, 'MyEcsCluster', {
            vpc: props.vpc,
        });

        const taskDefinition = new FargateTaskDefinition(this, 'MyTaskDef');
        taskDefinition.addContainer('MyContainer', {
            image: ContainerImage.fromEcrRepository(props.repository),
            memoryLimitMiB: 512,
            cpu: 256,
        });

        this.ecsService = new FargateService(this, 'MyFargateService', {
            cluster: this.ecsCluster,
            taskDefinition,
        });

        // Attach the ECS service to the blue target group initially
        this.ecsService.attachToApplicationTargetGroup(props.blueTargetGroup);
    }
}
