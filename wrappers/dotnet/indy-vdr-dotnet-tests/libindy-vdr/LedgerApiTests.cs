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
            string testSubmitterId = "LibindyDid111111111111";
            Dictionary<string, Dictionary<string, string>> testDict = new() { { "test", new Dictionary<string, string>() { { "description", "" } } } };
            string testAml = JsonConvert.SerializeObject(testDict);
            string testVersion = "1";
            string testAml_context = "test_aml_context";

            //Act
            uint testObject = await LedgerApi.BuildAcceptanceMechanismsRequestAsync(
                testSubmitterId,
                testAml,
                testVersion,
                testAml_context);

            //Assert
            testObject.Should().NotBe(0);
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
            testObject.Should().NotBe(0);
        }

        [Test, TestCase(TestName = "BuildSchemaRequestAsync call returns request handle.")]
        public async Task BuildSchemaRequestAsyncWorks()
        {
            //Arrange
            string testSubmitterId = "LibindyDid111111111111";
            Schema testSchema = new()
            {
                Id = "testId",
                Name = "testName",
                Version = "1.0",
                Ver = "1.0",
                AttrNames = new HashSet<string>() { "testAttribute1", "testAttribute2" },
                SeqNo = 5
            };

            string schemaJson = JsonConvert.SerializeObject(testSchema);

            //Act
            uint testObject = await LedgerApi.BuildSchemaRequestAsync(
                testSubmitterId,
                schemaJson);

            //Assert
            testObject.Should().NotBe(0);
        }
        [Test, TestCase(TestName = "BuildGetAttributeRequest call returns request handle.")]
        public async Task BuildGetAttributeRequestWorks()
        {
            //Arrange 
            string testSubmitterDid = "LibindyDid111111111111";
            string testTargetDid = "LibindyDid111111111111";
            string testHash = "";
            string testRaw = "";
            string testEnc = "";

            //Act
            uint testObject = await LedgerApi.BuildAttributeRequest(
                testTargetDid,
                testSubmitterDid);

            //Assert
            testObject.Should().NotBe(0);
        }
    }
}
