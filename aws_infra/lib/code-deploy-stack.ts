import * as cdk from 'aws-cdk-lib';
import {Construct} from 'constructs';
import {Cluster, FargateService} from 'aws-cdk-lib/aws-ecs';
import {Vpc} from 'aws-cdk-lib/aws-ec2';
import {
    ApplicationListener,
    ApplicationLoadBalancer,
    ApplicationTargetGroup
} from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import {EcsApplication, EcsDeploymentConfig, EcsDeploymentGroup} from 'aws-cdk-lib/aws-codedeploy';

interface CodeDeployStackProps extends cdk.StackProps {
    ecsService: FargateService;
    ecsCluster: Cluster;
    vpc: Vpc;
    alb: ApplicationLoadBalancer;
    listenerBlue: ApplicationListener;
    listenerGreen: ApplicationListener;
    targetGroupBlue: ApplicationTargetGroup;
    targetGroupGreen: ApplicationTargetGroup;
}

export class CodeDeployStack extends cdk.Stack {
    public readonly deploymentGroup: EcsDeploymentGroup;

    constructor(scope: Construct, id: string, props: CodeDeployStackProps) {
        super(scope, id, props);

        const application = new EcsApplication(this, 'EcsApplication');

        this.deploymentGroup = new EcsDeploymentGroup(this, 'EcsDeploymentGroup', {
            application,
            service: props.ecsService,
            deploymentConfig: EcsDeploymentConfig.ALL_AT_ONCE,
            blueGreenDeploymentConfig: {
                listener: props.listenerBlue,
                testListener: props.listenerGreen,
                blueTargetGroup: props.targetGroupBlue,
                greenTargetGroup: props.targetGroupGreen,
            },
            autoRollback: {
                failedDeployment: true,
            },
        });

        new cdk.CfnOutput(this, 'LoadBalancerDNS', {
            value: props.alb.loadBalancerDnsName,
        });
    }
}
