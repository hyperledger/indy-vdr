using FluentAssertions;
using indy_vdr_dotnet.libindy_vdr;
using indy_vdr_dotnet.models;
using Newtonsoft.Json;
using NUnit.Framework;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class LedgerApiTests
    {
        [Test, TestCase(TestName = "BuildAcceptanceMechanismsRequestAsync call returns request handle.")]
        public async Task BuildAcceptanceMechanismsRequestAsyncWorks()
        {
            //Arrange 
            string testSubmitter_id = "LibindyDid111111111111";
            string testAml = "{\"test\":{\"description\":\"testdescription\"}}";
            string testVersion = "1";
            string testAml_context = "test_aml_context";

            //Act
            uint testObject = await LedgerApi.BuildAcceptanceMechanismsRequestAsync(
                testSubmitter_id,
                testAml,
                testVersion,
                testAml_context);

            //Assert
            _ = testObject.Should().NotBe(0);
        }

        [Test, TestCase(TestName = "BuildGetAcceptanceMechanismsRequestAsync call returns request handle.")]
        public async Task BuildGetAcceptanceMechanismsRequestAsyncWorks()
        {
            //Arrange 
            long testTimestamp = DateTimeOffset.Now.ToUnixTimeSeconds();

            //Act
            uint testObject = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(
                testTimestamp);

            //Assert
            _ = testObject.Should().NotBe(0);
        }

        [Test, TestCase(TestName = "BuildRevocRegDefRequestAsync call returns request handle.")]
        public async Task BuildRevocRegDefRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterId = "LibindyDid111111111111";

            string revRegDefJson = "{\"id\":\"testId\",\"revocDefType\":\"CL_ACCUM\",\"tag\":\"testTag\",\"credDefId\":\"testCredDefId\",\"value\":{\"issuanceType\":\"ISSUANCE_BY_DEFAULT\",\"maxCredNum\":5,\"publicKeys\":{\"accumKey\":\"testAccumKey\"},\"tailsHash\":\"testTailsHash\",\"tailsLocation\":\"testTailsLocation\"},\"ver\":\"1.0\"}";

            //Act
            uint testObject = await LedgerApi.BuildRevocRegDefRequestAsync(
                testSubmitterId,
                revRegDefJson);

            //Assert
            _ = testObject.Should().NotBe(0);
        }

        [Test, TestCase(TestName = "BuildRevocRegEntryRequestAsync call returns request handle.")]
        public async Task BuildRevocRegEntryRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterId = "LibindyDid111111111111";
            string testRevRegDefId = "revreg:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:4:creddef:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:3:CL:schema:sov:did:sov:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:tag:CL_ACCUM:TAG_1";
            string testRevRegDefType = "CL_ACCUM";

            string deltaJson = "{\"ver\":\"1.0\",\"value\":\"test\"}";

            //Act
            uint testObject = await LedgerApi.BuildRevocRegEntryRequestAsync(
                testSubmitterId,
                testRevRegDefId,
                testRevRegDefType,
                deltaJson);

            //Assert
            _ = testObject.Should().NotBe(0);
        }

        [Test, TestCase(TestName = "BuildSchemaRequestAsync call returns request handle.")]
        public async Task BuildSchemaRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterId = "LibindyDid111111111111";

            string schemaJson = "{\"Handle\":0,\"id\":\"testId\",\"name\":\"testName\",\"version\":\"1.0\",\"attrNames\":[\"testAttribute1\",\"testAttribute2\"],\"ver\":\"1.0\",\"SeqNo\":5}";

            //Act
            uint testObject = await LedgerApi.BuildSchemaRequestAsync(
                testSubmitterId,
                schemaJson);

            //Assert
            _ = testObject.Should().NotBe(0);
        }
    }
}
