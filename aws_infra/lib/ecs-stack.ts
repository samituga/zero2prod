import * as cdk from 'aws-cdk-lib';
import {Construct} from 'constructs';
import {Cluster, ContainerImage, FargateService, FargateTaskDefinition, Protocol} from 'aws-cdk-lib/aws-ecs';
import {ApplicationTargetGroup} from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import {Repository} from 'aws-cdk-lib/aws-ecr';
import {Vpc} from 'aws-cdk-lib/aws-ec2';
import {DeploymentControllerType} from 'aws-cdk-lib/aws-ecs/lib/base/base-service';

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
            image: ContainerImage.fromEcrRepository(props.repository, 'latest'),
            essential: true,
            memoryLimitMiB: 512,
            cpu: 256,
            portMappings: [{
                containerPort: 8080,
                hostPort: 8080,
                protocol: Protocol.TCP,
            }],
            environment: {
                'APP_DATABASE__USERNAME': '',
                'APP_DATABASE__PASSWORD': '',
                'APP_DATABASE__HOST': '',
                'APP_DATABASE__PORT': '',
                'APP_DATABASE__DATABASE_NAME': '',
            }
        });

        this.ecsService = new FargateService(this, 'MyFargateService', {
            cluster: this.ecsCluster,
            desiredCount: 2,
            deploymentController: {type: DeploymentControllerType.CODE_DEPLOY},
            taskDefinition,
            assignPublicIp: true,
        });

        console.log("\n\n\n\n\ntaskDefinition.toString()\n\n\n\n\n");
        console.log(taskDefinition.toString());



        // Attach the ECS service to the blue target group initially
        this.ecsService.attachToApplicationTargetGroup(props.blueTargetGroup);
    }
}
