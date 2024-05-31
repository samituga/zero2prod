import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as rds from 'aws-cdk-lib/aws-rds';
import { Construct } from 'constructs';

interface RdsStackProps extends cdk.StackProps {
  vpc: ec2.Vpc;
}

export class RdsStack extends cdk.Stack {
  public readonly dbInstance: rds.DatabaseInstance;

  constructor(scope: Construct, id: string, props: RdsStackProps) {
    super(scope, id, props);

    this.dbInstance = new rds.DatabaseInstance(this, 'Zero2ProdDB', {
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_16_2,
      }),
      vpc: props.vpc,
      allocatedStorage: 20,
      maxAllocatedStorage: 40,
      instanceType: new cdk.aws_ec2.InstanceType('t3.micro'),
      multiAz: true,
      publiclyAccessible: false,
    });
  }
}
