using FluentAssertions;
using indy_vdr_dotnet;
using indy_vdr_dotnet.libindy_vdr;
using Newtonsoft.Json.Linq;
using NUnit.Framework;
using System;
using System.Threading.Tasks;

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
        [TestCase(TestName = "PrepareTxnAuthorAgreementAcceptance call throws when version is missing but text is given.")]
        public async Task PrepareTxnAuthorAgreementAcceptanceWorksWithTaaDigestThrows()
        {
            //Arrange
            string testAccMechType = "acc_mech_type";
            ulong testTime = (ulong)DateTimeOffset.Now.ToUnixTimeSeconds();
            string testTaaDigest = "taa_digest";
            string testText = "";

            //Act
            Func<Task> func = async () => await RequestApi.PrepareTxnAuthorAgreementAcceptanceAsync(
                testAccMechType,
                testTime,
                taaDigest: testTaaDigest,
                text: testText);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
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
            IntPtr requestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());
            
            //Act
            string requestBodyBeforeFree = await RequestApi.RequestGetBodyAsync(requestHandle);
            await RequestApi.RequestFreeAsync(requestHandle);
            Func<Task> act = async () => await RequestApi.RequestGetBodyAsync(requestHandle);

            //Assert
            requestBodyBeforeFree.Should().NotBe("");
            await act.Should().ThrowAsync<Exception>();
        }

        [Test]
        [TestCase(TestName = "RequestFree call with invalid pointer throws.")]
        public async Task RequestFreeThrows()
        {
            //Arrange
            IntPtr requestHandle = new();

            //Act
            Func<Task> func = async () => await RequestApi.RequestFreeAsync(requestHandle);


            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test]
        [TestCase(TestName = "RequestGetBody call returns a JSON string.")]
        public async Task RequestGetBodyWorks()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());

            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);

            //Assert
            requestBody.Should().NotBe("");
        }

        [Test]
        [TestCase(TestName = "RequestGetBody call with invalid pointer throws.")]
        public async Task RequestGetBodyThrows()
        {
            //Arrange
            IntPtr requestHandle = new();

            //Act
            Func<Task> func = async () => await RequestApi.RequestGetBodyAsync(requestHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test]
        [TestCase(TestName = "RequestGetSignatureInput call returns a signature string.")]
        public async Task RequestGetSignatureInputWorks()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());

            //Act
            string signature = await RequestApi.RequestGetSignatureInputAsync(requestHandle);

            //Assert
            signature.Should().NotBe("");
        }

        [Test]
        [TestCase(TestName = "RequestGetSignatureInput call with invalid pointer throws.")]
        public async Task RequestGetSignatureInputThrows()
        {
            //Arrange
            IntPtr requestHandle = new();

            //Act
            Func<Task> func = async () => await RequestApi.RequestGetSignatureInputAsync(requestHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test]
        [TestCase(TestName = "RequestSetEndorser call sets the endorser.")]
        public async Task RequestSetEndorserWorks()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
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
        [TestCase(TestName = "RequestSetEndorser call with invalid pointer throws.")]
        public async Task RequestSetEndorserThrows()
        {
            //Arrange
            IntPtr requestHandle = new();
            string testEndorser = "Endorser11111111111111";
            //Act
            JObject requestBodyJObj = new();
            Func<Task> func = async () => await RequestApi.RequestSetEndorserAsync(
                requestHandle,
                testEndorser);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test]
        [TestCase(TestName = "RequestSetMultiSignature call sets a multi-signature entry.")]
        public async Task RequestSetMultiSignatureWorks()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
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
        [TestCase(TestName = "RequestSetMultiSignature call with invalid pointer throws.")]
        public async Task RequestSetMultiSignatureThrows()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject requestBodyJObj = JObject.Parse(requestBody);

            string testIdentifier = "V4SGRU86Z58d6TV7PBUe6f";
            string testMultiSig = "sig";
            Func<Task> func = async () => await RequestApi.RequestSetMultiSignatureAsync(
                new IntPtr(),
                testIdentifier,
                testMultiSig);
            string actual = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test]
        [TestCase(TestName = "RequestSetSiganture call sets a signature entry.")]
        public async Task RequestSetSigantureWorks()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
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
        [TestCase(TestName = "RequestSetSiganture call with invalid pointer throws.")]
        public async Task RequestSetSigantureThrows()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject requestBodyJObj = JObject.Parse(requestBody);

            string testMultiSig = "{\"signature\":\"sig\"}";
            Func<Task> func = async () => await RequestApi.RequestSetSigantureAsync(
                new IntPtr(),
                testMultiSig);
            string actual = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject actualJObj = JObject.Parse(actual);
            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }

        [Test]
        [TestCase(TestName = "RequestSetTxnAuthorAgreementAcceptance call sets a signature entry.")]
        public async Task RequestSetTxnAuthorAgreementAcceptanceWorks()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
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

        [Test]
        [TestCase(TestName = "RequestSetTxnAuthorAgreementAcceptance call with invalid pointer throws.")]
        public async Task RequestSetTxnAuthorAgreementAcceptanceThrows()
        {
            //Arrange
            IntPtr requestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testTaaAcceptance = "{\"mechanism\":\"acc_mech_type\",\"taaDigest\":\"taa_digest\",\"time\":1655683200}";
            //Act
            string requestBody = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject requestBodyJObj = JObject.Parse(requestBody);

            Func<Task> func = async () => await RequestApi.RequestSetTxnAuthorAgreementAcceptanceAsync(
                new IntPtr(),
                testTaaAcceptance);
            string actual = await RequestApi.RequestGetBodyAsync(requestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
    }
}
