import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import { Construct } from 'constructs';
import { EcsConfig, RdsConfig } from '../config/type';

export interface Props extends cdk.StackProps {
  ecsConfig: EcsConfig;
  rdsConfig: RdsConfig;
  vpc: ec2.Vpc;
}

export class SgStack extends cdk.Stack {
  readonly alb: ec2.SecurityGroup;
  readonly ecs: ec2.SecurityGroup;
  readonly rds: ec2.SecurityGroup;

  constructor(scope: Construct, id: string, props: Props) {
    super(scope, id, props);

    const { ecsConfig, rdsConfig, vpc } = props;

    this.alb = new ec2.SecurityGroup(this, 'AlbSecurityGroup', {
      vpc,
      description: 'Allow HTTP and HTTPS traffic to load balancer',
      allowAllOutbound: true,
    });

    this.ecs = new ec2.SecurityGroup(this, 'EcsSecurityGroup', {
      vpc,
      allowAllOutbound: true,
    });

    this.rds = new ec2.SecurityGroup(this, 'RdsSecurityGroup', {
      vpc,
      allowAllOutbound: false,
    });

    this.albRules(ecsConfig);
    this.ecsRules(ecsConfig, rdsConfig);
    this.rdsRules(vpc, rdsConfig);
  }

  private albRules(ecsConfig: EcsConfig) {
    this.alb.addIngressRule(ec2.Peer.anyIpv4(), ec2.Port.tcp(80), 'allow HTTP traffic from anywhere');
    // this.alb.addIngressRule(ec2.Peer.anyIpv4(), ec2.Port.tcp(443), 'allow HTTPS traffic from anywhere'); TODO
    // this.alb.addEgressRule(this.ecs, ec2.Port.tcp(ecsConfig.taskDefConfig.containerPort));
  }

  private ecsRules(ecsConfig: EcsConfig, rdsConfig: RdsConfig) {
    this.ecs.addIngressRule(this.alb, ec2.Port.tcp(ecsConfig.taskDefConfig.containerPort));
    // this.ecs.addEgressRule(this.rds, ec2.Port.tcp(rdsConfig.port));
  }

  private rdsRules(vpc: ec2.Vpc, rdsConfig: RdsConfig) {
    this.rds.addIngressRule(this.ecs, ec2.Port.tcp(rdsConfig.port));
    this.rds.addIngressRule(
      ec2.Peer.ipv4(vpc.vpcCidrBlock),
      ec2.Port.tcp(rdsConfig.port),
      'Allow RDS access from within the VPC',
    );
  }
}
