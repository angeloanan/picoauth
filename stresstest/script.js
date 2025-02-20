import http from "k6/http";
import faker from "k6/x/faker";
import { check, sleep } from "k6";

export const options = {
    // A number specifying the number of VUs to run concurrently.
    vus: 10,
    // A string specifying the total duration of the test run.
    duration: "30s",
};

export function setup() {
    /**
     * @type {Map<string, string>} Credentials
     */
    const creds = new Map();

    return { credentials: creds };
}

const isRegisteringUser = Math.random() > 0.9;
// VU Code
export default function (sharedData) {
    const username = faker.internet.username();
    const password = faker.internet.password({
        length: 8 + Math.round(Math.random() * 93), // 8+, up to 100
        memorable: Math.random() > 0.5, // 50% chance
    });
    const displayName = faker.internet.displayName();

    // Should user be registering
    if (isRegisteringUser) {
        const registerData = {
            username,
            password,
            display_name: displayName,
        };

        const res = http.post(
            "http://localhost:3000/auth/register",
            JSON.stringify(registerData),
            {
                headers: { "Content-Type": "application/json" },
                tags: {
                    name: isSuccessLogin ? "ValidRegister" : "InvalidRegister",
                },
            },
        );

        check(res, {
            "register response code was 200": (res) => res.status == 200,
            "register response contains {success: true}": (res) =>
                JSON.parse(res.body).success === true,
        });

        sharedData.credentials.set(username, password);

        // Sleep for 2 sec minimum, up to 12 secs
        sleep(2 + Math.random() * 10);
    }

    // Login flow
    const res = http.post(
        "http://localhost:3000/auth/login",
        JSON.stringify(data),
        {
            headers: { "Content-Type": "application/json" },
            tags: { name: isSuccessLogin ? "ValidLogin" : "InvalidLogin" },
        },
    );

    if (isRegisteringUser) {
        check(res, {
            "response code was 200": (res) => res.status == 200,
            "response contains a body": (res) => res.body != null,
            "response body has access token": (res) =>
                JSON.parse(res.body).access_token != null,
            "response body has refresh token": (res) =>
                JSON.parse(res.body).refresh_token != null,
        });
    } else {
        check(res, {
            "response code was 401": (res) => res.status == 401,
            "response contains a body": (res) => res.body != null,
            "response body has error": (res) =>
                JSON.parse(res.body)?.error != null,
            "response body errors is proper": (res) =>
                JSON.parse(res.body)?.error === "Invalid username or password",
        });
    }
}

export function teardown(data) {}
