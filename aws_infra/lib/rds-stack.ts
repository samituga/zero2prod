import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as rds from 'aws-cdk-lib/aws-rds';
import { Construct } from 'constructs';
import { RdsConfig } from '../config/type';

interface RdsStackProps extends cdk.StackProps {
  config: RdsConfig;
  vpc: ec2.Vpc;
}

export class RdsStack extends cdk.Stack {
  public readonly dbInstance: rds.DatabaseInstance;

  constructor(scope: Construct, id: string, props: RdsStackProps) {
    super(scope, id, props);

    const { config, vpc } = props;

    this.dbInstance = new rds.DatabaseInstance(this, 'Zero2ProdDB', {
      vpc,
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_16_2,
      }),
      allocatedStorage: config.allocatedStorage,
      maxAllocatedStorage: config.maxAllocatedStorage,
      instanceType: new cdk.aws_ec2.InstanceType(config.instanceType),
      multiAz: true,
      publiclyAccessible: false,
    });
  }
}
