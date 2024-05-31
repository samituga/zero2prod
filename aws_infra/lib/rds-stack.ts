import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as rds from 'aws-cdk-lib/aws-rds';
import { Construct } from 'constructs';
import { RdsConfig } from '../config/type';

interface RdsStackProps extends cdk.StackProps {
  config: RdsConfig;
  vpc: ec2.Vpc;
  sg: ec2.SecurityGroup;
}

export class RdsStack extends cdk.Stack {
  public readonly dbInstance: rds.DatabaseInstance;

  constructor(scope: Construct, id: string, props: RdsStackProps) {
    super(scope, id, props);

    const { config, vpc, sg } = props;

    const subnetGroup = new rds.SubnetGroup(this, 'RdsSubnetGroup', {
      vpc,
      vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_ISOLATED },
      description: 'Subnet group for the RDS instance',
    });

    this.dbInstance = new rds.DatabaseInstance(this, 'RdsInstance', {
      allocatedStorage: config.allocatedStorage,
      maxAllocatedStorage: config.maxAllocatedStorage,
      instanceType: new cdk.aws_ec2.InstanceType(config.instanceType),
      port: config.port,
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_16_2,
      }),
      multiAz: true,
      publiclyAccessible: false,

      vpc,
      securityGroups: [sg],
      subnetGroup,
    });
  }
}
