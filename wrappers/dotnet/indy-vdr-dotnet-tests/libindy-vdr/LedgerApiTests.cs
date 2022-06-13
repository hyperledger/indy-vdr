using FluentAssertions;
using indy_vdr_dotnet.libindy_vdr;
using NUnit.Framework;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
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

    }
}
