import * as cdk from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import { Construct } from 'constructs';
import { VpcConfig } from '../config/type';

interface VpcProps extends cdk.StackProps {
  config: VpcConfig;
}

export class VpcStack extends cdk.Stack {
  public readonly vpc: ec2.Vpc;

  constructor(scope: Construct, id: string, props: VpcProps) {
    super(scope, id, props);

    const { config } = props;

    this.vpc = new ec2.Vpc(this, 'Zero2ProdVPC', {
      maxAzs: config.maxAzs,
      subnetConfiguration: [
        {
          subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
          name: 'PrivateIsolatedSubnet',
        },
        {
          subnetType: ec2.SubnetType.PUBLIC,
          name: 'PublicSubnet',
        },
      ],
    });
  }
}
