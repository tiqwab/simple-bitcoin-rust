import {Button as BButton, Container as BContainer, Form as BForm} from "react-bootstrap";
import React from "react";
import axios, {AxiosResponse} from "axios";

function Home() {
    return (
        <BContainer>
            <CurrentBalance />
            <SendCoinForm />
        </BContainer>
    )
}

interface GetBalanceResponse {
    balance: number,
}

interface BalanceState {
    data: GetBalanceResponse | null,
    error: any,
    isLoading: boolean,
}

function CurrentBalance() {
    const [result, setResult] = React.useState<BalanceState>({ data: null, error: null, isLoading: true });
    const base_url = process.env.REACT_APP_SIMPLE_BITCOIN_BASE_URL

    const fetcher = (apiPath: string) => {
        const url = `${base_url}${apiPath}`;
        axios.get(url).then((resp: AxiosResponse<GetBalanceResponse>) => {
            setResult((cur) => ({
                ...cur,
                data: resp.data,
                isLoading: false,
            }))
        }).catch((err) => {
            setResult((cur) => ({
                ...cur,
                error: err,
                isLoading: false,
            }))
        });
    };

    React.useEffect(() => {
        setResult(() => ({
            data: null,
            error: null,
            isLoading: true,
        }))

        fetcher("/balance");
    }, []);

    return (
        <div className="mt-3 mb-3">
            Current Balance: { result.data !== null ? result.data.balance : 0 }
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