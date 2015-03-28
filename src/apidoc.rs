pub mod client {
    extern crate hyper;
    use rustc_serialize::json;
    use super::models;

    pub struct Code {
        base_url: String,
        token: String
    }

    impl Code {
        pub fn new(base_url: String, token: String) -> Code {
            Code {
                base_url: base_url,
                token: token
            }
        }

        pub fn get_by_organization_key_and_application_key_and_version_and_generator_key(
            &self,
            organization_key: &str,
            application_key: &str,
            version: &str,
            generator_key: &str
        ) -> hyper::HttpResult<hyper::client::Response> {
            let mut client = hyper::client::Client::new();
            let mut url = self.base_url.clone();
            url.push('/');
            url.push_str(organization_key);
            url.push('/');
            url.push_str(application_key);
            url.push('/');
            url.push_str(version);
            url.push('/');
            url.push_str(generator_key);
            let scheme = hyper::header::Basic {
                username: self.token.clone(),
                password: None
            };
            client.get(&url[..])
                .header(hyper::header::Authorization(scheme))
                .send()
        }
    }

    pub struct Validations {
        base_url: String
    }

    impl Validations {
        pub fn new(base_url: String) -> Validations {
            Validations {
                base_url: base_url
            }
        }

        pub fn post(&self, value: &str) -> hyper::HttpResult<hyper::client::Response> {
            let mut client = hyper::client::Client::new();
            let mut url = self.base_url.clone();
            url.push_str("/validations");
            client.post(&url[..]).body(value).send()
        }
    }

    pub struct Versions {
        base_url: String,
        token: String
    }

    impl Versions {
        pub fn new(base_url: String, token: String) -> Versions {
            Versions {
                base_url: base_url,
                token: token
            }
        }

        pub fn put_by_organization_key_and_application_key_and_version(
            &self,
            organization_key: &str,
            application_key: &str,
            version: &str,
            version_form: models::VersionForm
        ) -> hyper::HttpResult<hyper::client::Response> {
            let mut client = hyper::client::Client::new();
            let mut url = self.base_url.clone();
            url.push('/');
            url.push_str(organization_key);
            url.push('/');
            url.push_str(application_key);
            url.push('/');
            url.push_str(version);
            let json = json::encode(&version_form).unwrap();
            let scheme = hyper::header::Basic {
                username: self.token.clone(),
                password: None
            };
            client.put(&url[..]).body(&json[..])
                .header(hyper::header::Authorization(scheme))
                .header(hyper::header::ContentType(application_json()))
                .send()
        }
    }

    fn application_json() -> hyper::mime::Mime {
        "application/json".parse().unwrap()
    }
}

pub mod models {
    extern crate chrono;
    extern crate uuid;

    use rustc_serialize::Decodable;
    use rustc_serialize::Decoder;
    use rustc_serialize::Encodable;
    use rustc_serialize::Encoder;
    use std;

    /**
     * An application has a name and multiple versions of its API.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Application {
        pub guid: uuid::Uuid,
        pub organization: Reference,
        pub name: String,
        pub key: String,
        pub visibility: Visibility,
        pub description: Option<String>
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct ApplicationForm {
        pub name: String,
        pub key: Option<String>,
        pub description: Option<String>,
        pub visibility: Visibility
    }

    /* TODO no rustc-serialize support in chrono
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Audit {
        pub createdAt: chrono::DateTime<chrono::UTC>,
        pub createdBy: ReferenceGuid,
        pub updatedAt: chrono::DateTime<chrono::UTC>,
        pub updatedBy: ReferenceGuid
    }
    */

    /**
     * Separate resource used only for the few actions that require the full token.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct CleartextToken {
        pub token: String
    }

    /**
     * Generated source code.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Code {
        pub generator: Generator,
        pub source: String
    }

    /**
     * Represents a single domain name (e.g. www.apidoc.me). When a new user registers
     * and confirms their email, we automatically associate that user with a member of
     * the organization associated with their domain. For example, if you confirm your
     * account with an email address of foo@gilt.com, we will automatically create a
     * membership request on your behalf to join the organization with domain gilt.com.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Domain {
        pub name: String
    }

    /**
     * Data used to confirm an email address. The token is an internal unique
     * identifier used to lookup the specific email address and user account for which
     * we sent an email verification email.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct EmailVerificationConfirmationForm {
        pub token: String
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Error {
        pub code: String,
        pub message: String
    }

    /**
     * An apidoc generator
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Generator {
        pub guid: uuid::Uuid,
        pub key: String,
        pub uri: String,
        pub name: String,
        pub language: Option<String>,
        pub description: Option<String>,
        pub visibility: Visibility,
        pub owner: User,
        pub enabled: bool
    }

    /**
     * Form to create a new generator
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct GeneratorCreateForm {
        pub key: String,
        pub uri: String,
        pub visibility: Visibility
    }

    /**
     * Form to enable or disable a generator for an organization
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct GeneratorOrgForm {
        pub enabled: bool
    }

    /**
     * Form to update a generator
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct GeneratorUpdateForm {
        pub visibility: Option<Visibility>,
        pub enabled: Option<bool>
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Healthcheck {
        pub status: String
    }

    /**
     * A membership represents a user in a specific role to an organization.
     * Memberships cannot be created directly. Instead you first create a membership
     * request, then that request is either accepted or declined.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Membership {
        pub guid: uuid::Uuid,
        pub user: User,
        pub organization: Organization,
        pub role: String
    }

    /**
     * A membership request represents a user requesting to join an organization with a
     * specific role (e.g. as a member or an admin). Membership requests can be
     * reviewed by any current admin of the organization who can either accept or
     * decline the request.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct MembershipRequest {
        pub guid: uuid::Uuid,
        pub user: User,
        pub organization: Organization,
        pub role: String
    }

    /**
     * An organization is used to group a set of applications together.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Organization {
        pub guid: uuid::Uuid,
        pub key: String,
        pub name: String,
        pub namespace: String,
        pub visibility: Visibility,
        pub domains: Vec<Domain>
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct OrganizationForm {
        pub name: String,
        pub key: Option<String>,
        pub namespace: String,
        pub visibility: Option<Visibility>,
        pub domains: Vec<String>
    }

    /**
     * Represents the original input used to create an application version
     */
    pub struct Original {
        pub original_type: OriginalType,
        pub data: String
    }

    impl Encodable for Original {
        fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
            e.emit_struct("original", 2, |e| {
                try!(e.emit_struct_field("type", 0, |e| self.original_type.encode(e)));
                try!(e.emit_struct_field("data", 1, |e| self.data.encode(e)));
                Ok(())
            })
        }
    }

    impl Decodable for Original {
        fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
            d.read_struct("original", 2, |d| {
                let original_type: OriginalType = try!(
                    d.read_struct_field("type", 0, OriginalType::decode));
                let data: String = try!(
                    d.read_struct_field("data", 1, |d| d.read_str()));
                Ok(Original {
                    original_type: original_type,
                    data: data
                })
            })
        }
    }

    pub struct OriginalForm {
        pub original_type: Option<OriginalType>,
        pub data: String
    }

    impl Encodable for OriginalForm {
        fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
            e.emit_struct("original_form", 2, |e| {
                try!(e.emit_struct_field("type", 0, |e| self.original_type.encode(e)));
                try!(e.emit_struct_field("data", 1, |e| self.data.encode(e)));
                Ok(())
            })
        }
    }

    impl Decodable for OriginalForm {
        fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
            d.read_struct("original_form", 2, |d| {
                let original_type: Option<OriginalType> = try!(
                    d.read_struct_field("type", 0, Option::<OriginalType>::decode));
                let data: String = try!(
                    d.read_struct_field("data", 1, |d| d.read_str()));
                Ok(OriginalForm {
                    original_type: original_type,
                    data: data
                })
            })
        }
    }

    /**
     * Allows a user to change their password with authentication from a token.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct PasswordReset {
        pub token: String,
        pub password: String
    }

    /**
     * Create a password reset request - e.g. an email containing a one time URL to
     * change a password
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct PasswordResetRequest {
        pub email: String
    }

    /**
     * On a successful password reset, return some metadata about the user modified.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct PasswordResetSuccess {
        pub user_guid: uuid::Uuid
    }

    /**
     * Represents a reference to another model.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Reference {
        pub guid: uuid::Uuid,
        pub key: String
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct ReferenceGuid {
        pub guid: uuid::Uuid
    }

    /**
     * Represents a user that is currently subscribed to a publication
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Subscription {
        pub guid: uuid::Uuid,
        pub organization: Organization,
        pub user: User,
        pub publication: Publication
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct SubscriptionForm {
        pub organization_key: String,
        pub user_guid: uuid::Uuid,
        pub publication: Publication
    }

    /* TODO can't (de)serialize Audit
    /**
     * A token gives a user access to the API.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Token {
        pub guid: uuid::Uuid,
        pub user: User,
        pub maskedToken: String,
        pub description: Option<String>,
        pub audit: Audit
    }
    */

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct TokenForm {
        pub user_guid: uuid::Uuid,
        pub description: Option<String>
    }

    /**
     * A user is a top level person interacting with the api doc server.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct User {
        pub guid: uuid::Uuid,
        pub email: String,
        pub nickname: String,
        pub name: Option<String>
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct UserForm {
        pub email: String,
        pub password: String,
        pub nickname: Option<String>,
        pub name: Option<String>
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct UserUpdateForm {
        pub email: String,
        pub nickname: String,
        pub name: Option<String>
    }

    /**
     * Used only to validate json files - used as a resource where http status code
     * defines success
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Validation {
        pub valid: bool,
        pub errors: Vec<String>
    }

    /**
     * Represents a unique version of the application.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Version {
        pub guid: uuid::Uuid,
        pub organization: Reference,
        pub application: Reference,
        pub version: String,
        pub original: Option<Original>,
        // TODO pub service: com.gilt.apidoc.spec.v0.models.Service
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct VersionForm {
        pub original_form: OriginalForm,
        pub visibility: Option<Visibility>
    }

    /**
     * Users can watch individual applications which enables features like receiving an
     * email notification when there is a new version of an application.
     */
    #[derive(RustcEncodable, RustcDecodable)]
    pub struct Watch {
        pub guid: uuid::Uuid,
        pub user: User,
        pub organization: Organization,
        pub application: Application
    }

    #[derive(RustcEncodable, RustcDecodable)]
    pub struct WatchForm {
        pub user_guid: uuid::Uuid,
        pub organization_key: String,
        pub application_key: String
    }

    pub enum OriginalType {

        /**
         * The original is in the api.json format
         */
        ApiJson,

        /**
         * The original in the swagger.json format
         */
        SwaggerJson,

        /**
         * The original is in Avro Idl format
         */
        AvroIdl,

        /**
         * UNDEFINED captures values that are sent either in error or
         * that were added by the server after this library was
         * generated. We want to make it easy and obvious for users of
         * this library to handle this case gracefully.
         *
         * We use all CAPS for the variable name to avoid collisions
         * with the camel cased values above.
         */
        UNDEFINED(String)
    }

    impl Encodable for OriginalType {
        fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
            match self {
                &OriginalType::ApiJson => e.emit_str("api_json"),
                &OriginalType::SwaggerJson => e.emit_str("swagger_json"),
                &OriginalType::AvroIdl => e.emit_str("avro_idl"),
                &OriginalType::UNDEFINED(ref value) => e.emit_str(&value)
            }
        }
    }

    impl Decodable for OriginalType {
        fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
            d.read_str().map(|value| {
                match &value[..] {
                    "api_json" => OriginalType::ApiJson,
                    "swagger_json" => OriginalType::SwaggerJson,
                    "avro_idl" => OriginalType::AvroIdl,
                    _ => OriginalType::UNDEFINED(value)
                }
            })
        }
    }

    /**
     * A publication represents something that a user can subscribe to. An example
     * would be subscribing to an email alert whenever a new version of an application
     * is created.
     */
    pub enum Publication {

        /**
         * For organizations for which I am an administrator, email me whenever a user
         * applies to join the org.
         */
        MembershipRequestsCreate,

        /**
         * For organizations for which I am a member, email me whenever a user joins the
         * org.
         */
        MembershipsCreate,

        /**
         * For organizations for which I am a member, email me whenever an application is
         * created.
         */
        ApplicationsCreate,

        /**
         * For applications that I watch, email me whenever a version is created.
         */
        VersionsCreate,

        /**
         * UNDEFINED captures values that are sent either in error or
         * that were added by the server after this library was
         * generated. We want to make it easy and obvious for users of
         * this library to handle this case gracefully.
         *
         * We use all CAPS for the variable name to avoid collisions
         * with the camel cased values above.
         */
        UNDEFINED(String)
    }

    impl Encodable for Publication {
        fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
            match self {
                &Publication::MembershipRequestsCreate => e.emit_str("membership_requests.create"),
                &Publication::MembershipsCreate => e.emit_str("memberships.create"),
                &Publication::ApplicationsCreate => e.emit_str("applications.create"),
                &Publication::VersionsCreate => e.emit_str("versions.create"),
                &Publication::UNDEFINED(ref value) => e.emit_str(value)
            }
        }
    }

    impl Decodable for Publication {
        fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
            d.read_str().map(|value| {
                match &value[..] {
                    "membership_requests.create" => Publication::MembershipRequestsCreate,
                    "memberships.create" => Publication::MembershipsCreate,
                    "applications.create" => Publication::ApplicationsCreate,
                    "versions.create" => Publication::VersionsCreate,
                    _ => Publication::UNDEFINED(value)
                }
            })
        }
    }

    /**
     * Controls who is able to view this version
     */
    #[derive(Clone, Debug)]
    pub enum Visibility {

        /**
         * Only the creator can view this application
         */
        User,

        /**
         * Any member of the organization can view this application
         */
        Organization,

        /**
         * Anybody, including non logged in users, can view this application
         */
        Public,

        /**
         * UNDEFINED captures values that are sent either in error or
         * that were added by the server after this library was
         * generated. We want to make it easy and obvious for users of
         * this library to handle this case gracefully.
         *
         * We use all CAPS for the variable name to avoid collisions
         * with the camel cased values above.
         */
        UNDEFINED(String)
    }

    impl Visibility {
        pub fn valid(&self) -> Result<&Self, &Self> {
            match self {
                &Visibility::UNDEFINED(_) => Err(self),
                _ => Ok(self)
            }
        }
    }

    impl std::fmt::Display for Visibility {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                &Visibility::User => f.write_str("user"),
                &Visibility::Organization => f.write_str("organization"),
                &Visibility::Public => f.write_str("public"),
                &Visibility::UNDEFINED(ref value) => f.write_str(value)
            }
        }
    }

    impl ::rustc_serialize::Encodable for Visibility {
        fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
            match self {
                &Visibility::User => e.emit_str("user"),
                &Visibility::Organization => e.emit_str("organization"),
                &Visibility::Public => e.emit_str("public"),
                &Visibility::UNDEFINED(ref value) => e.emit_str(value)
            }
        }
    }

    impl ::rustc_serialize::Decodable for Visibility {
        fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
            d.read_str().map(|value| {
                match &value[..] {
                    "user" => Visibility::User,
                    "organization" => Visibility::Organization,
                    "public" => Visibility::Public,
                    _ => Visibility::UNDEFINED(value)
                }
            })
        }
    }
}
