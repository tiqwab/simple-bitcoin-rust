import React from "react";
import axios, {AxiosResponse} from "axios";

const base_url = process.env.REACT_APP_SIMPLE_BITCOIN_BASE_URL

interface Result<T, > {
    data: T | undefined,
    error: any | undefined,
    isLoading: boolean,
}

const useFetcher = <T, >(apiPath: string) => {
    const [result, setResult] = React.useState<Result<T>>({data: undefined, error: undefined, isLoading: false});

    React.useEffect(() => {
        const url = `${base_url}${apiPath}`;

        setResult((cur) => ({
            ...cur,
            isLoading: true,
        }));

        axios.get(url).then((resp: AxiosResponse<T>) => {
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
    }, [apiPath]);

    return result;
};

export default useFetcher;