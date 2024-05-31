import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as elb from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Construct } from 'constructs';
import { AlbConfig, EcsConfig } from '../config/type';

interface AlbStackProps extends cdk.StackProps {
  config: AlbConfig,
  ecsConfig: EcsConfig,
  vpc: ec2.Vpc;
}

export class AlbStack extends cdk.Stack {
  public readonly alb: elb.ApplicationLoadBalancer;
  public readonly listener80: elb.ApplicationListener;
  public readonly targetGroupBlue: elb.ApplicationTargetGroup;
  public readonly targetGroupGreen: elb.ApplicationTargetGroup;

  constructor(scope: Construct, id: string, props: AlbStackProps) {
    super(scope, id, props);

    const { config, ecsConfig, vpc } = props;

    this.alb = new elb.ApplicationLoadBalancer(this, 'LB', {
      vpc,
      internetFacing: true,
    });

    this.listener80 = this.alb.addListener('BlueListener', {
      port: 80,
      protocol: elb.ApplicationProtocol.HTTP,
    });

    const healthCheckConfig = config.healthCheck;

    const targetGroupProps = {
      vpc,
      port: ecsConfig.taskDefConfig.hostPort,
      protocol: elb.ApplicationProtocol.HTTP,
      targets: [],
      targetType: elb.TargetType.IP,

      healthCheck: {
        path: healthCheckConfig.path,
        interval: cdk.Duration.seconds(healthCheckConfig.intervalSec),
        timeout: cdk.Duration.seconds(healthCheckConfig.timeoutSec),
        healthyThresholdCount: healthCheckConfig.healthyThresholdCount,
        unhealthyThresholdCount: healthCheckConfig.unhealthyThresholdCount,
        healthyHttpCodes: healthCheckConfig.healthyHttpCodes,
      },
    };

    this.targetGroupBlue = new elb.ApplicationTargetGroup(this, 'BlueTargetGroup', targetGroupProps);
    this.targetGroupGreen = new elb.ApplicationTargetGroup(this, 'GreenTargetGroup', targetGroupProps);

    this.listener80.addTargetGroups('GreenTargetGroup', {
      targetGroups: [this.targetGroupGreen],
    });

    new cdk.CfnOutput(this, 'LoadBalancerDNS', {
      value: this.alb.loadBalancerDnsName,
    });
  }
}
