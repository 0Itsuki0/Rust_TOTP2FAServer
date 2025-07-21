async function register() {
    const email = document.getElementById("email").value
    const password = document.getElementById("password").value
    console.log("register for ", email, password)

    const url = "/auth/register"
    try {
        const response = await fetch(url, {
            method: "POST",
            body: JSON.stringify({
                email: email,
                password: password,
            }),
            headers: {
                "Content-Type": "application/json"
            }
        })

        try {
            const json = await response.json()

            console.log(json)
            if (!response.ok) {
                alert(`Error: Response status: ${response.status}, message: ${json.message}`)
                return
            }

            document.getElementById("signedInEmail").innerText = json.user.email
            document.getElementById("signedInContainer").style.display = "block"
            document.getElementById("formContainer").style.display = "none"

            if (json.user.otp_enabled == true) {
                document.getElementById("enableOTPButton").style.display = "none"
                document.getElementById("disableOTPButton").style.display = "block"
            } else {
                document.getElementById("enableOTPButton").style.display = "block"
                document.getElementById("disableOTPButton").style.display = "none"
            }

        } catch {
            const text = response.text()
            console.log(text)
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}, message: ${text}`)
            }
        }
    } catch (error) {
        alert(error.message)
    }
}

async function signIn() {
    const email = document.getElementById("email").value
    const password = document.getElementById("password").value
    console.log("signin for ", email, password)

    const url = "/auth/signin"
    try {
        const response = await fetch(url, {
            method: "POST",
            body: JSON.stringify({
                email: email,
                password: password,
            }),
            headers: {
                "Content-Type": "application/json"
            }
        })

        try {
            const json = await response.json()

            console.log(json)
            if (!response.ok) {
                alert(`Error: Response status: ${response.status}, message: ${json.message}`)
                return
            }

            if (json.otp_verification_required == false) {
                document.getElementById("signedInEmail").innerText = json.user.email
                document.getElementById("signedInContainer").style.display = "block"
                document.getElementById("formContainer").style.display = "none"

                if (json.user.otp_enabled == true) {
                    document.getElementById("enableOTPButton").style.display = "none"
                    document.getElementById("disableOTPButton").style.display = "block"
                } else {
                    document.getElementById("enableOTPButton").style.display = "block"
                    document.getElementById("disableOTPButton").style.display = "none"
                }
                return
            }

            document.getElementById("otpQRCode").src = ""
            document.getElementById("otpQRCode").style.display = "none"
            document.getElementById("otpTokenContainer").style.display = "block"

        } catch {
            const text = response.text()
            console.log(text)
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}, message: ${text}`)
            }
        }
    } catch (error) {
        alert(error.message)
    }
}

async function verifyOTPToken() {
    const otpToken = document.getElementById("otpToken").value
    console.log("verify token: ", otpToken)

    const url = "/auth/otp/verify"
    try {
        const response = await fetch(url, {
            method: "POST",
            body: JSON.stringify({
                otp_token: otpToken,
            }),
            headers: {
                "Content-Type": "application/json"
            }
        })

        try {
            const json = await response.json()

            console.log(json)
            if (!response.ok) {
                alert(`Error: Response status: ${response.status}, message: ${json.message}`)
                return
            }

            const isValid = json.otp_verified
            if (!isValid) {
                alert("Invalid token. Please try again.")
                return
            }

            document.getElementById("signedInEmail").innerText = json.user.email
            document.getElementById("signedInContainer").style.display = "block"
            document.getElementById("formContainer").style.display = "none"
            document.getElementById("otpTokenContainer").style.display = "none"

            document.getElementById("enableOTPButton").style.display = "none"
            document.getElementById("disableOTPButton").style.display = "block"

            document.getElementById("otpToken").value = ""

        } catch {
            const text = response.text()
            console.log(text)
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}, message: ${text}`)
            }
        }
    } catch (error) {
        alert(error.message)
    }

}

async function SignOut() {
    const url = "/auth/signout"
    try {
        const response = await fetch(url, {
            method: "GET"
        })

        try {
            const json = await response.json()

            console.log(json)

            if (!response.ok) {
                alert(`Error: Response status: ${response.status}, message: ${json.message}`)
                return
            }

            document.getElementById("signedInEmail").innerText = ""
            document.getElementById("signedInContainer").style.display = "none"
            document.getElementById("otpTokenContainer").style.display = "none"
            document.getElementById("formContainer").style.display = "block"

        } catch {
            const text = response.text()
            console.log(text)
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}, message: ${text}`)
            }
        }
    } catch (error) {
        alert(error.message)
    }

}

async function enableOTP() {
    const url = "/auth/otp/enable?response_type=QR_BASE64"
    try {
        const response = await fetch(url, {
            method: "GET"
        })

        try {
            const json = await response.json()

            console.log(json)

            if (!response.ok) {
                alert(`Error: Response status: ${response.status}, message: ${json.message}`)
                return
            }

            document.getElementById("otpQRCode").src = `data:image/png;base64, ${json.otp_qr_base64}`
            document.getElementById("otpQRCode").style.display = "block"
            document.getElementById("otpTokenContainer").style.display = "block"

        } catch {
            const text = response.text()
            console.log(text)
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}, message: ${text}`)
            }
        }
    } catch (error) {
        alert(error.message)
    }

}

async function disableOTP() {
    const url = "/auth/otp/disable"
    try {
        const response = await fetch(url, {
            method: "GET"
        })

        try {
            const json = await response.json()

            console.log(json)

            if (!response.ok) {
                alert(`Error: Response status: ${response.status}, message: ${json.message}`)
                return
            }

            document.getElementById("enableOTPButton").style.display = "block"
            document.getElementById("disableOTPButton").style.display = "none"

        } catch {
            const text = response.text()
            console.log(text)
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}, message: ${text}`)
            }
        }
    } catch (error) {
        alert(error.message)
    }

}