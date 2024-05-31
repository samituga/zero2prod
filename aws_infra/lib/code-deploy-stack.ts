import * as cdk from 'aws-cdk-lib';
import * as codedeploy from 'aws-cdk-lib/aws-codedeploy';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as elb from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { Construct } from 'constructs';

interface CodeDeployStackProps extends cdk.StackProps {
  ecsService: ecs.FargateService;
  ecsCluster: ecs.Cluster;
  vpc: ec2.Vpc;
  alb: elb.ApplicationLoadBalancer;
  listenerBlue: elb.ApplicationListener;
  listenerGreen: elb.ApplicationListener;
  targetGroupBlue: elb.ApplicationTargetGroup;
  targetGroupGreen: elb.ApplicationTargetGroup;
}

export class CodeDeployStack extends cdk.Stack {
  public readonly deploymentGroup: codedeploy.EcsDeploymentGroup;

  constructor(scope: Construct, id: string, props: CodeDeployStackProps) {
    super(scope, id, props);

    const application = new codedeploy.EcsApplication(this, 'EcsApplication');

    this.deploymentGroup = new codedeploy.EcsDeploymentGroup(this, 'EcsDeploymentGroup', {
      application,
      service: props.ecsService,
      deploymentConfig: codedeploy.EcsDeploymentConfig.ALL_AT_ONCE,
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
