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
        #region Tests for PrepareTxnAuthorAgreementAcceptanceAsync
        [Test, TestCase(TestName = "PrepareTxnAuthorAgreementAcceptance call with taa_digest returns a JSON.")]
        public async Task PrepareTxnAuthorAgreementAcceptanceWorksWithTaaDigest()
        {
            //Arrange
            string testAccMechType = "acc_mech_type";
            ulong testTime = (ulong)DateTimeOffset.Now.ToUnixTimeSeconds();
            string testTaaDigest = "taa_digest";

            //Act
            string actual = await RequestApi.PrepareTxnAuthorAgreementAcceptanceAsync(
                testAccMechType,
                testTime,
                taaDigest: testTaaDigest);

            //Assert
            _ = actual.Should().NotBe("");
        }

        [Test, TestCase(TestName = "PrepareTxnAuthorAgreementAcceptance call throws when version is missing but text is given.")]
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
        #endregion

        #region Tests for PrepareTxnAuthorAgreementAcceptanceAsync
        [Test, TestCase(TestName = "PrepareTxnAuthorAgreementAcceptance call with version, text returns a JSON.")]
        public async Task PrepareTxnAuthorAgreementAcceptanceWorksWithVersionText()
        {
            //Arrange
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
            _ = actual.Should().NotBe("");
        }
        #endregion

        #region Tests for RequestFreeAsync
        [Test, TestCase(TestName = "RequestFree call frees given testRequestHandle.")]
        public async Task RequestFreeWorks()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());

            //Act
            string testRequestBodyBeforeFree = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            await RequestApi.RequestFreeAsync(testRequestHandle);
            Func<Task> act = async () => await RequestApi.RequestGetBodyAsync(testRequestHandle);

            //Assert
            _ = testRequestBodyBeforeFree.Should().NotBe("");
            await act.Should().ThrowAsync<Exception>();
        }

        [Test, TestCase(TestName = "RequestFree call with invalid pointer throws.")]
        public async Task RequestFreeThrows()
        {
            //Arrange
            IntPtr testRequestHandle = new();

            //Act
            Func<Task> func = async () => await RequestApi.RequestFreeAsync(testRequestHandle);


            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for RequestGetBodyAsync
        [Test, TestCase(TestName = "RequestGetBody call returns a JSON string.")]
        public async Task RequestGetBodyWorks()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());

            //Act
            string testRequestBody = await RequestApi.RequestGetBodyAsync(testRequestHandle);

            //Assert
            _ = testRequestBody.Should().NotBe("");
        }

        [Test, TestCase(TestName = "RequestGetBody call with invalid pointer throws.")]
        public async Task RequestGetBodyThrows()
        {
            //Arrange
            IntPtr testRequestHandle = new();

            //Act
            Func<Task> func = async () => await RequestApi.RequestGetBodyAsync(testRequestHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for RequestGetSignatureInputAsync
        [Test, TestCase(TestName = "RequestGetSignatureInput call returns a signature string.")]
        public async Task RequestGetSignatureInputWorks()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetAcceptanceMechanismsRequestAsync(DateTimeOffset.Now.ToUnixTimeSeconds());

            //Act
            string signature = await RequestApi.RequestGetSignatureInputAsync(testRequestHandle);

            //Assert
            _ = signature.Should().NotBe("");
        }

        [Test, TestCase(TestName = "RequestGetSignatureInput call with invalid pointer throws.")]
        public async Task RequestGetSignatureInputThrows()
        {
            //Arrange
            IntPtr testRequestHandle = new();

            //Act
            Func<Task> func = async () => await RequestApi.RequestGetSignatureInputAsync(testRequestHandle);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for RequestSetEndorserAsync
        [Test, TestCase(TestName = "RequestSetEndorser call sets the endorser.")]
        public async Task RequestSetEndorserWorks()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testEndorser = "Endorser11111111111111";
            string testRequestBody = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject testRequestBodyJObj = JObject.Parse(testRequestBody);

            //Act
            await RequestApi.RequestSetEndorserAsync(
                testRequestHandle,
                testEndorser);
            string actual = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            testRequestBodyJObj.Should().NotContainKey("endorser");
            actualJObj.Should().ContainKey("endorser");
        }

        [Test, TestCase(TestName = "RequestSetEndorser call with invalid pointer throws.")]
        public async Task RequestSetEndorserThrows()
        {
            //Arrange
            IntPtr testRequestHandle = new();
            string testEndorser = "Endorser11111111111111";
            JObject testRequestBodyJObj = new();

            //Act
            Func<Task> func = async () => await RequestApi.RequestSetEndorserAsync(
                testRequestHandle,
                testEndorser);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for RequestSetMultiSignatureAsync

        [Test, TestCase(TestName = "RequestSetMultiSignature call sets a multi-signature entry.")]
        public async Task RequestSetMultiSignatureWorks()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testRequestBody = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject testRequestBodyJObj = JObject.Parse(testRequestBody);
            string testIdentifier = "V4SGRU86Z58d6TV7PBUe6f";
            string testMultiSig = "sig";

            //Act
            await RequestApi.RequestSetMultiSignatureAsync(
                testRequestHandle,
                testIdentifier,
                testMultiSig);
            string actual = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            testRequestBodyJObj.Should().NotContainKey("signatures");
            actualJObj.Should().ContainKey("signatures");
        }

        [Test, TestCase(TestName = "RequestSetMultiSignature call with invalid pointer throws.")]
        public async Task RequestSetMultiSignatureThrows()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testtestRequestBody = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject testtestRequestBodyJObj = JObject.Parse(testtestRequestBody);
            string testIdentifier = "V4SGRU86Z58d6TV7PBUe6f";
            string testMultiSig = "sig";

            //Act
            Func<Task> func = async () => await RequestApi.RequestSetMultiSignatureAsync(
                new IntPtr(),
                testIdentifier,
                testMultiSig);
            string actual = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for RequestSetSigantureAsync
        [Test]
        [TestCase(TestName = "RequestSetSiganture call sets a signature entry.")]
        public async Task RequestSetSigantureWorks()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testtestRequestBody = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject testtestRequestBodyJObj = JObject.Parse(testtestRequestBody);
            string testMultiSig = "{\"signature\":\"sig\"}";

            //Act
            await RequestApi.RequestSetSigantureAsync(
                testRequestHandle,
                testMultiSig);
            string actual = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            testtestRequestBodyJObj.Should().NotContainKey("signature");
            actualJObj.Should().ContainKey("signature");
        }

        [Test, TestCase(TestName = "RequestSetSiganture call with invalid pointer throws.")]
        public async Task RequestSetSigantureThrows()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testtestRequestBody = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject testtestRequestBodyJObj = JObject.Parse(testtestRequestBody);
            string testMultiSig = "{\"signature\":\"sig\"}";

            //Act
            Func<Task> func = async () => await RequestApi.RequestSetSigantureAsync(
                new IntPtr(),
                testMultiSig);
            string actual = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion

        #region Tests for RequestSetTxnAuthorAgreementAcceptanceAsync
        [Test, TestCase(TestName = "RequestSetTxnAuthorAgreementAcceptance call sets a signature entry.")]
        public async Task RequestSetTxnAuthorAgreementAcceptanceWorks()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testTaaAcceptance = "{\"mechanism\":\"acc_mech_type\",\"taaDigest\":\"taa_digest\",\"time\":1655683200}";
            string testRequestBody = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject testRequestBodyJObj = JObject.Parse(testRequestBody);

            //Act
            await RequestApi.RequestSetTxnAuthorAgreementAcceptanceAsync(
                testRequestHandle,
                testTaaAcceptance);
            string actual = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            testRequestBodyJObj.Should().NotContainKey("taaAcceptance");
            actualJObj.Should().ContainKey("taaAcceptance");
        }

        [Test, TestCase(TestName = "RequestSetTxnAuthorAgreementAcceptance call with invalid pointer throws.")]
        public async Task RequestSetTxnAuthorAgreementAcceptanceThrows()
        {
            //Arrange
            IntPtr testRequestHandle = await LedgerApi.BuildGetTxnRequestAsync(1, 1);
            string testTaaAcceptance = "{\"mechanism\":\"acc_mech_type\",\"taaDigest\":\"taa_digest\",\"time\":1655683200}";
            //Act
            string testRequestBody = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject testRequestBodyJObj = JObject.Parse(testRequestBody);

            Func<Task> func = async () => await RequestApi.RequestSetTxnAuthorAgreementAcceptanceAsync(
                new IntPtr(),
                testTaaAcceptance);
            string actual = await RequestApi.RequestGetBodyAsync(testRequestHandle);
            JObject actualJObj = JObject.Parse(actual);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
        }
        #endregion
    }
}
