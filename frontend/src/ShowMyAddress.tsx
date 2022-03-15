import {Container as BContainer} from "react-bootstrap";
import React from "react";
import useFetcher from "./useFetcher";

interface GetMyAddressResponse {
    address: string,
}

const ShowMyAddress = () => {
    const result = useFetcher<GetMyAddressResponse>("/address/me");

    return (
        <BContainer>
            <div className="mt-3 mb-3 text-break">
                My Address: {result.data?.address ?? ""}
            </div>
        </BContainer>
    );
};

export default ShowMyAddress;