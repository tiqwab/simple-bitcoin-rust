import {Button as BButton, Container as BContainer, Form as BForm} from "react-bootstrap";
import React, {ChangeEvent, FormEvent} from "react";
import axios from "axios";
import {useToast} from "./useToast";
import useFetcher from "./useFetcher";

const base_url = process.env.REACT_APP_SIMPLE_BITCOIN_BASE_URL

function Home() {
    return (
        <BContainer>
            <CurrentBalance/>
            <SendCoinForm/>
        </BContainer>
    )
}

interface GetBalanceResponse {
    balance: number,
}

function CurrentBalance() {
    const result = useFetcher<GetBalanceResponse>("/balance");

    return (
        <div className="mt-3 mb-3">
            Current Balance: {result.data?.balance ?? 0}
        </div>
    )
}

interface SendCoinRequest {
    recipient: string | undefined,
    value: number,
    fee: number,
}

function SendCoinForm() {
    const [params, setParams] = React.useState<SendCoinRequest>({recipient: undefined, value: 0, fee: 0})
    const showToast = useToast();
    const formRef = React.useRef<HTMLFormElement>(null);

    const handleChangePayTo = React.useCallback((ev: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
        ev.preventDefault()
        let recipient = ev.currentTarget.value;
        setParams((cur) => ({...cur, recipient: recipient}))
    }, []);

    const handleChangeValue = React.useCallback((ev: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
        ev.preventDefault()
        let value = parseInt(ev.currentTarget.value);
        setParams((cur) => ({...cur, value: value}))
    }, []);

    const handleChangeFee = React.useCallback((ev: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
        ev.preventDefault()
        let fee = parseInt(ev.currentTarget.value);
        setParams((cur) => ({...cur, fee: fee}))
    }, []);

    const handleSubmit = React.useCallback((ev: FormEvent<HTMLFormElement>) => {
        ev.preventDefault()
        let url = `${base_url}/transaction`
        axios.post(url, params).then((resp) => {
            showToast({text: "Sent coins successfully", type: "success"});
            if (formRef.current !== null) {
                formRef.current.reset();
            }
        }).catch((err) => {
            showToast({text: "Failed to send coins", type: "failure"});
            console.log(`Failed to send coins: ${err}`);
        })
    }, [params, showToast]);

    return (
        <BForm onSubmit={handleSubmit} ref={formRef}>
            <BForm.Group className="mb-3" controlId="sendCoinFormPayTo">
                <BForm.Label>Pay to</BForm.Label>
                <BForm.Control type="text" placeholder="Enter recipient address" required onChange={handleChangePayTo}/>
            </BForm.Group>

            <BForm.Group className="mb-3" controlId="sendCoinFormValue">
                <BForm.Label>Value</BForm.Label>
                <BForm.Control type="number" placeholder="Enter value (Bitcoin)" required onChange={handleChangeValue}/>
            </BForm.Group>
            <BForm.Group className="mb-3" controlId="sendCoinFormFee">
                <BForm.Label>Fee</BForm.Label>
                <BForm.Control type="number" placeholder="Enter fee (Bitcoin)" required onChange={handleChangeFee}/>
            </BForm.Group>
            <BButton variant="primary" type="submit">
                Send Coin(s)
            </BButton>
        </BForm>
    )
}

export default Home;