import * as cdk from 'aws-cdk-lib';
import {Construct} from 'constructs';
import {Vpc} from 'aws-cdk-lib/aws-ec2';
import {
    ApplicationListener,
    ApplicationLoadBalancer,
    ApplicationProtocol,
    ApplicationTargetGroup,
    TargetType
} from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import {Duration} from "aws-cdk-lib/core/lib/duration";
import {ApplicationTargetGroupProps} from "aws-cdk-lib/aws-elasticloadbalancingv2/lib/alb/application-target-group";
import {BaseApplicationListenerProps} from "aws-cdk-lib/aws-elasticloadbalancingv2/lib/alb/application-listener";

interface AlbStackProps extends cdk.StackProps {
    vpc: Vpc;
}

export class AlbStack extends cdk.Stack {
    public readonly alb: ApplicationLoadBalancer;
    public readonly prodListener: ApplicationListener;
    public readonly testListener: ApplicationListener;
    public readonly blueTargetGroup: ApplicationTargetGroup;
    public readonly greenTargetGroup: ApplicationTargetGroup;

    constructor(scope: Construct, id: string, props: AlbStackProps) {
        super(scope, id, props);

        this.alb = new ApplicationLoadBalancer(this, 'LB', {
            vpc: props.vpc,
            internetFacing: true,
        });

        const listenerProps: BaseApplicationListenerProps = {
            port: 80,
            protocol: ApplicationProtocol.HTTP,
        };

        this.prodListener = this.alb.addListener('ProdListener', listenerProps);
        this.testListener = this.alb.addListener('TestListener', listenerProps);


        const targetGroupProps: ApplicationTargetGroupProps = {
            vpc: props.vpc,
            port: 80,
            protocol: ApplicationProtocol.HTTP,
            targetType: TargetType.IP,

            healthCheck: {
                path: "/health_check",
                interval: Duration.seconds(30),
                timeout: Duration.seconds(5),
                healthyThresholdCount: 5,
                unhealthyThresholdCount: 2,
                healthyHttpCodes: "200",
            }
        };

        this.blueTargetGroup = new ApplicationTargetGroup(this, 'BlueTargetGroup', targetGroupProps);
        this.greenTargetGroup = new ApplicationTargetGroup(this, 'GreenTargetGroup', targetGroupProps);

        this.prodListener.addTargetGroups('BlueTargetGroup', {
            targetGroups: [this.blueTargetGroup],
        });

        this.testListener.addTargetGroups('GreenTargetGroup', {
            targetGroups: [this.greenTargetGroup],
        });

        new cdk.CfnOutput(this, 'LoadBalancerDNS', {
            value: this.alb.loadBalancerDnsName,
        });
    }
}
