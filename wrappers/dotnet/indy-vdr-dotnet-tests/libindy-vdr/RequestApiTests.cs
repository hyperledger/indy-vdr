using FluentAssertions;
using indy_vdr_dotnet.libindy_vdr;
using Newtonsoft.Json.Linq;
using NUnit.Framework;
using System;
using System.Threading.Tasks;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class RequestApiTests
    {
        [Test]
        [TestCase(TestName = "PrepareTxnAuthorAgreementAcceptance call with taa_digest returns a JSON.")]
        public async Task PrepareTxnAuthorAgreementAcceptanceWorksWithTaaDigest()
        {
            //Arrange
            string expected = "";
            string testAccMechType = "acc_mech_type";
            ulong testTime = (ulong) DateTimeOffset.Now.ToUnixTimeSeconds();
            string testTaaDigest = "taa_digest";
            //Act

            string actual = await RequestApi.PrepareTxnAuthorAgreementAcceptanceAsync(
                testAccMechType,
                testTime,
                taaDigest: testTaaDigest);

            //Assert
            actual.Should().NotBe(expected);
        }
        [Test]
        [TestCase(TestName = "PrepareTxnAuthorAgreementAcceptance call with version, text returns a JSON.")]
        public async Task PrepareTxnAuthorAgreementAcceptanceWorksWithVersionText()
        {
            //Arrange
            string expected = "";
            string testAccMechType = "acc_mech_type";
            ulong testTime = (ulong)DateTimeOffset.Now.ToUnixTimeSeconds();
            string testText = "text";
            string testVersion = "version";
            //Act

            string actual = await RequestApi.PrepareTxnAuthorAgreementAcceptanceAsync(
                testAccMechType,
                testTime,
                text: testText,
                version: testVersion);

            //Assert
            actual.Should().NotBe(expected);
        }

        [Test]
        [TestCase(TestName = "RequestFree call frees given RequestHandle.")]
        public async Task RequestFreeWorks()
        {
            //Arrange
            uint requestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());
            
            //Act
            string requestBodyBeforeFree = await RequestApi.RequestGetBodyAsync(requestHandle);
            await RequestApi.RequestFreeAsync(requestHandle);
            string actual = await RequestApi.RequestGetBodyAsync(requestHandle);

            //Assert
            requestBodyBeforeFree.Should().NotBe("");
            actual.Should().Be("");
        }

        [Test]
        [TestCase(TestName = "RequestGetBody call returns a JSON string.")]
        public async Task RequestGetBodyWorks()
        {
            //Arrange
            uint requestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());

            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);

            //Assert
            requestBody.Should().NotBe("");
        }

        [Test]
        [TestCase(TestName = "RequestGetSignatureInput call returns a signature string.")]
        public async Task RequestGetSignatureInputWorks()
        {
            //Arrange
            uint requestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());

            //Act
            string signature = await RequestApi.RequestGetSignatureInputAsync(requestHandle);

            //Assert
            signature.Should().NotBe("");
        }

        [Test]
        [TestCase(TestName = "RequestSetEndorser call sets the endorser.")]
        public async Task RequestSetEndorserWorks()
        {
            //Arrange
            uint requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testEndorser = "Endorser11111111111111";
            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject requestBodyJObj = JObject.Parse(requestBody);
            await RequestApi.RequestSetEndorserAsync(
                requestHandle,
                testEndorser);
            string actual = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject actualJObj = JObject.Parse(actual);
            //Assert
            requestBodyJObj.Should().NotContainKey("endorser");
            actualJObj.Should().ContainKey("endorser");
        }

        [Test]
        [TestCase(TestName = "RequestSetMultiSignature call sets a multi-signature entry.")]
        public async Task RequestSetMultiSignatureWorks()
        {
            //Arrange
            uint requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject requestBodyJObj = JObject.Parse(requestBody);

            string testIdentifier = "V4SGRU86Z58d6TV7PBUe6f";
            string testMultiSig = "sig";
            await RequestApi.RequestSetMultiSignatureAsync(
                requestHandle,
                testIdentifier,
                testMultiSig);
            string actual = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject actualJObj = JObject.Parse(actual);
            //Assert
            requestBodyJObj.Should().NotContainKey("signatures");
            actualJObj.Should().ContainKey("signatures");
        }

        [Test]
        [TestCase(TestName = "RequestSetSiganture call sets a signature entry.")]
        public async Task RequestSetSigantureWorks()
        {
            //Arrange
            uint requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject requestBodyJObj = JObject.Parse(requestBody);

            string testMultiSig = "{\"signature\":\"sig\"}";
            await RequestApi.RequestSetSigantureAsync(
                requestHandle,
                testMultiSig);
            string actual = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject actualJObj = JObject.Parse(actual);
            //Assert
            requestBodyJObj.Should().NotContainKey("signature");
            actualJObj.Should().ContainKey("signature");
        }

        [Test]
        [TestCase(TestName = "RequestSetTxnAuthorAgreementAcceptance call sets a signature entry.")]
        public async Task RequestSetTxnAuthorAgreementAcceptanceWorks()
        {
            //Arrange
            uint requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testTaaAcceptance = "{\"mechanism\":\"acc_mech_type\",\"taaDigest\":\"taa_digest\",\"time\":1655683200}";
            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject requestBodyJObj = JObject.Parse(requestBody);

            await RequestApi.RequestSetTxnAuthorAgreementAcceptanceAsync(
                requestHandle,
                testTaaAcceptance);
            string actual = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject actualJObj = JObject.Parse(actual);
            //Assert
            requestBodyJObj.Should().NotContainKey("taaAcceptance");
            actualJObj.Should().ContainKey("taaAcceptance");
        }
    }
}
