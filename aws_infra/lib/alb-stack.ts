import * as cdk from 'aws-cdk-lib';
import {Duration} from 'aws-cdk-lib';
import {Construct} from 'constructs';
import {Vpc} from 'aws-cdk-lib/aws-ec2';
import {
    ApplicationListener,
    ApplicationLoadBalancer,
    ApplicationProtocol,
    ApplicationTargetGroup,
    TargetType
} from 'aws-cdk-lib/aws-elasticloadbalancingv2';

interface AlbStackProps extends cdk.StackProps {
    vpc: Vpc;
}

export class AlbStack extends cdk.Stack {
    public readonly alb: ApplicationLoadBalancer;
    public readonly listenerBlue: ApplicationListener;
    public readonly listenerGreen: ApplicationListener;
    public readonly targetGroupBlue: ApplicationTargetGroup;
    public readonly targetGroupGreen: ApplicationTargetGroup;

    constructor(scope: Construct, id: string, props: AlbStackProps) {
        super(scope, id, props);

        this.alb = new ApplicationLoadBalancer(this, 'LB', {
            vpc: props.vpc,
            internetFacing: true,
        });

        this.listenerBlue = this.alb.addListener('BlueListener', {
            port: 80,
            protocol: ApplicationProtocol.HTTP,
        });

        this.listenerGreen = this.alb.addListener('GreenListener', {
            port: 8080,
            protocol: ApplicationProtocol.HTTP
        });

        const targetGroupProps = {
            vpc: props.vpc,
            port: 80,
            protocol: ApplicationProtocol.HTTP,
            targetType: TargetType.IP,

            healthCheck: {
                path: '/health_check',
                interval: Duration.seconds(30),
                timeout: Duration.seconds(5),
                healthyThresholdCount: 5,
                unhealthyThresholdCount: 2,
                healthyHttpCodes: '200',
            },
        };

        this.targetGroupBlue = new ApplicationTargetGroup(this, 'BlueTargetGroup', targetGroupProps);
        this.targetGroupGreen = new ApplicationTargetGroup(this, 'GreenTargetGroup', targetGroupProps);

        this.listenerBlue.addTargetGroups('GreenTargetGroup', {
            targetGroups: [this.targetGroupGreen],
        });

        this.listenerGreen.addTargetGroups('BlueTargetGroup', {
            targetGroups: [this.targetGroupBlue],
        });

        new cdk.CfnOutput(this, 'LoadBalancerDNS', {
            value: this.alb.loadBalancerDnsName,
        });
    }
}
