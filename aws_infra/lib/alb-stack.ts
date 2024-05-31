import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as elb from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Construct } from 'constructs';

interface AlbStackProps extends cdk.StackProps {
  vpc: ec2.Vpc;
}

export class AlbStack extends cdk.Stack {
  public readonly alb: elb.ApplicationLoadBalancer;
  public readonly listenerBlue: elb.ApplicationListener;
  public readonly listenerGreen: elb.ApplicationListener;
  public readonly targetGroupBlue: elb.ApplicationTargetGroup;
  public readonly targetGroupGreen: elb.ApplicationTargetGroup;

  constructor(scope: Construct, id: string, props: AlbStackProps) {
    super(scope, id, props);

    const { vpc } = props;

    this.alb = new elb.ApplicationLoadBalancer(this, 'LB', {
      vpc,
      internetFacing: true,
    });

    this.listenerBlue = this.alb.addListener('BlueListener', {
      port: 80,
      protocol: elb.ApplicationProtocol.HTTP,
    });

    this.listenerGreen = this.alb.addListener('GreenListener', {
      port: 8080,
      protocol: elb.ApplicationProtocol.HTTP,
    });

    const targetGroupProps = {
      vpc,
      port: 80,
      protocol: elb.ApplicationProtocol.HTTP,
      targetType: elb.TargetType.IP,

      healthCheck: {
        path: '/health_check',
        interval: cdk.Duration.seconds(30),
        timeout: cdk.Duration.seconds(5),
        healthyThresholdCount: 5,
        unhealthyThresholdCount: 2,
        healthyHttpCodes: '200',
      },
    };

    this.targetGroupBlue = new elb.ApplicationTargetGroup(this, 'BlueTargetGroup', targetGroupProps);
    this.targetGroupGreen = new elb.ApplicationTargetGroup(this, 'GreenTargetGroup', targetGroupProps);

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
