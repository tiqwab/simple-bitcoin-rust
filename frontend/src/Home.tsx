import {Button as BButton, Container as BContainer, Form as BForm} from "react-bootstrap";
import React from "react";

function Home() {
    return (
        <BContainer>
            <CurrentBalance />
            <SendCoinForm />
        </BContainer>
    )
}

function CurrentBalance() {
    const [balance, setBalance] = React.useState(0);

    return (
        <div className="mt-3 mb-3">
            Current Balance: { balance }
        </div>
    )
}

function SendCoinForm() {
    return (
        <BForm>
            <BForm.Group className="mb-3" controlId="sendCoinFormPayTo">
                <BForm.Label>Pay to</BForm.Label>
                <BForm.Control type="text" placeholder="Enter recipient address" />
            </BForm.Group>

            <BForm.Group className="mb-3" controlId="sendCoinFormValue">
                <BForm.Label>Value</BForm.Label>
                <BForm.Control type="number" placeholder="Enter value (Bitcoin)" />
            </BForm.Group>
            <BForm.Group className="mb-3" controlId="sendCoinFormFee">
                <BForm.Label>Fee</BForm.Label>
                <BForm.Control type="number" placeholder="Enter fee (Bitcoin)" />
            </BForm.Group>
            <BButton variant="primary" type="submit">
                Send Coin(s)
            </BButton>
        </BForm>
    )
}

export default Home;