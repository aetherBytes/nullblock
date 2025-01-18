# import pandas as pd
#
#
# def process_and_clean_data(data):
#     # Convert raw data to DataFrame
#     df = pd.DataFrame(data)
#
#     # Example cleaning: filter transactions above 1 SOL
#     df["amount"] = df["amount"].apply(lambda x: x / 1e9)  # Convert lamports to SOL
#     cleaned_df = df[df["amount"] > 1]  # Threshold can be adjusted
#
#     return cleaned_df
#
#
# def serialize_data(df):
#     return df.to_json(orient="records")
