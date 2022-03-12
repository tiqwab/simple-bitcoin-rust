import {Button as BButton, Container as BContainer, Form as BForm} from "react-bootstrap";
import React from "react";
import axios from "axios";

const UpdateBlockchain = ({
                              onUpdateBlockchainSuccess,
                              onUpdateBlockchainFailure
                          }: { onUpdateBlockchainSuccess: () => void, onUpdateBlockchainFailure: () => void }) => {
    const handleSubmit = React.useCallback(async () => {
        const url = `http://localhost:12345/update-balance`;
        axios.post(url).then((resp) => {
            onUpdateBlockchainSuccess()
        }).catch((err) => {
            onUpdateBlockchainFailure()
        }).finally(() => {
        })
    }, [onUpdateBlockchainSuccess, onUpdateBlockchainFailure]);

    return (
        <BContainer>
            <UpdateBlockchainForm onSubmit={handleSubmit}/>
        </BContainer>
    )
}

function UpdateBlockchainForm({onSubmit}: { onSubmit: () => void }) {
    const handleSubmit = React.useCallback(async (ev) => {
        ev.preventDefault();
        onSubmit();
    }, [onSubmit])

    return (
        <BForm className="mt-3 mb-3">
            <BButton variant="primary" type="submit" onClick={handleSubmit}>
                Update Blockchain
            </BButton>
        </BForm>
    )
}

export default UpdateBlockchain;